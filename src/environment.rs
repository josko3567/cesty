use crate::{
    error::ErrorPosition, 
    filegroup::FileGroup,
    lister::ListerFile, environment::modify::Modification
};

use std::{
    fmt::Display, 
    ptr::null_mut, path::Path,
};

use colored::Colorize;
use indoc::formatdoc;
use clang_sys::*;

use self::modify::{Position, Modify};

#[derive(Debug, Clone)]
pub enum Error {

    // FileInPool(ErrorPosition, String),
    CannotOpenFile(ErrorPosition, String, String),
    ImproperInclude(ErrorPosition, String, String, modify::Position)

}

impl Error {

    pub fn code(&self) -> String {
        return format!("E;{:X}:{:X}", 
            FileGroup::from(filename!()
            ) as u8, 
            unsafe { *(self as *const Self as *const u8) }
        );
    }
    
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::CannotOpenFile(pos, f,err ) => {
                fmtperr!(pos,
                "Cannot open file!",
                "
                    Cannot open file...
                        {}
                    ... due to the following error...
                        {}
                ",
                    fmterr_val!(f),
                    err.bold()
                )}
            Self::ImproperInclude(pos, inc, file, filepos) => {
                fmtperr!(pos,
                    "Bad #include!",
                    "
                        In file...
                            {}
                        ...on line {} the inclusion directive...
                            {}
                        ...is incorrectly written!
                    ",
                        fmterr_val!(file),
                        filepos.line,
                        fmterr_val!(inc)
                )
            }
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

/// A environment for placing `main()` into
/// depending on the standalone value.
/// 
/// `full` contains the entire unaltered file.
/// 
/// `bodyclean` contains the entire file without function bodies.
#[derive(Debug)]
pub struct Environment {
    
    pub full:      String,
    pub clean: String
    
}


pub(super) mod modify {

    use crate::error::ErrorPosition;

    use std::{
        ops::Range,
        sync::Mutex,
        ffi::{CStr, c_uint, OsString}, 
        ptr::null_mut,
        path::Path
    };

    use clang_sys::*;
    use itertools::Itertools;
    use lazy_static::lazy_static;

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(super) enum Type {
        Morph,
        Remove
    }

    #[derive(Clone, Debug)]
    pub(super) struct Position {
        pub(super) range:  Range<usize>,
        pub(super) line:   usize,
        pub(super) column: usize
    }

    #[derive(Clone, Debug)]
    pub(super) struct IncludeInfo {
        pub(super) filepath: String
    }

    #[derive(Clone, Debug)]
    pub(super) enum Modification {
        Include(Type, Position, IncludeInfo),
        FunctionBody(Type, Position)
    }

    lazy_static!(
        static ref STACK: Mutex<Vec<Modification>> = Mutex::new(vec![]);
    );

    fn position_from_cursor(
        cursor: CXCursor
    ) -> Position 
    { unsafe {

        let range_cx = clang_getCursorExtent(cursor);

        let mut range_start: u32 = 0;
        let mut range_end:   u32 = 0;
        let mut line:        u32 = 0;
        let mut column:      u32 = 0;

        clang_getFileLocation(
            clang_getRangeStart(range_cx), 
            null_mut(),
            std::ptr::addr_of_mut!(line) as *mut c_uint,
            std::ptr::addr_of_mut!(column) as *mut c_uint,
            std::ptr::addr_of_mut!(range_start) as *mut c_uint
        );

        clang_getFileLocation(
            clang_getRangeEnd(range_cx), 
            null_mut(),
            null_mut(),
            null_mut(), 
            std::ptr::addr_of_mut!(range_end) as *mut c_uint
        );

        Position { 
            range: (range_start as usize)..(range_end as usize), 
            line:   line as usize,
            column: column as usize
        }

    }}

    /// Function that gets passed to [`clang_visitChildren`],
    /// that extracts [`crate::Environment`] ranges into the
    /// [`crate::offset::stack`] function.
    /// After [`clang_visitChildren`] is finished you can extract all the
    /// environment ranges from [`crate::offset::stack`] with 
    /// `offset::Opt::PushAll`.
    #[allow(non_snake_case)]
    extern "C" fn filter(

        ccur: CXCursor,
        _parent: CXCursor,
        _data: CXClientData

    ) -> i32 
    { unsafe {
        
        // Filter to find function bodies in current file.
        if clang_Location_isFromMainFile(
            clang_getCursorLocation(ccur))
            == 0 
        {
            
            return CXChildVisit_Continue;  
            
        }
        else if clang_getCursorKind(ccur) == CXCursor_InclusionDirective
        {

            let filepath: String = {
                let file: CXFile = clang_getIncludedFile(ccur);
                let filepath_cx: CXString = clang_getFileName(file);
                let filepath_rust = CStr::from_ptr(filepath_cx.data as *mut i8)
                    .to_string_lossy()
                    .to_string();
                clang_disposeString(filepath_cx);
                filepath_rust
            };

            STACK.lock().unwrap().push(
                Modification::Include(
                    Type::Morph, 
                    position_from_cursor(ccur), 
                    IncludeInfo { filepath }
                )
            );

            return CXChildVisit_Continue;

        }
        else if clang_getCursorKind(ccur) 
            != CXCursor_CompoundStmt // Compund statment is brackets => { ... }
        || clang_getCursorKind(clang_getCursorSemanticParent(ccur)) 
            != CXCursor_FunctionDecl
        {
            
            return CXChildVisit_Recurse;
            
        }
        else
        {
            
            STACK.lock().unwrap().push(
                Modification::FunctionBody(
                    Type::Remove,
                    position_from_cursor(ccur)
                )
            );

            return CXChildVisit_Continue;
            
        }

    }}

    pub(super) fn find_from_cursor(
        cursor: CXCursor
    ) -> Vec<Modification> 
    {

        STACK.lock().unwrap().clear();
            
        unsafe { clang_visitChildren(
            cursor,
            filter,
            null_mut()
        )};

        STACK.lock().unwrap().reverse();

        return STACK.lock().unwrap().clone();

    }

    pub(super) trait Modify {
        fn modify(
            &mut self,
            filename: &OsString,
            modification: &Modification, 
            allowed: Vec<Type>
        ) -> Result<(), super::Error>;
    }

    impl Modify for String {
        fn modify(
            &mut self, 
            filename: &OsString,
            modification: &Modification, 
            allowed: Vec<Type>
        ) -> Result<(), super::Error>{
            match modification {
                Modification::Include(ty, pos, info) => {
                    if allowed.contains(&ty) {

                        
                        // FOR SOME REASON, UNBEKNOWNST TO ALL OF HUMANITY
                        // directive_start WILL ALWAYS BE ZERO WITH THE COMMENTED
                        // OUT CODE, BUT THE SCUFFED VERSION WORKS PERFECTLY???!??!
                        /*
                        let Some(mut directive_start) = 
                        self[(&pos.range).start+1..(&pos.range).end].chars()
                            .find_position(|x: &char| !x.is_ascii_whitespace())
                        else {
                            reterr!(
                                super::Error::ImproperInclude,
                                self[pos.range.clone()].to_string(),
                                filename.to_string_lossy().to_string(),
                                pos.to_owned()
                            );
                        };
                        */
                        let mut directive_start = {
                            let mut i: usize = 1;
                            for ch 
                            in self[(&pos.range).start+i..(&pos.range).end].chars()
                            {
                                if ch.is_ascii_whitespace() {
                                    i+=1
                                } else {
                                    break;
                                }
                            }
                            if self[(&pos.range).start+i..(&pos.range).end]
                            .chars()
                            .try_len()
                            .is_ok_and(|x| x == i)
                            {
                                reterr!(
                                    super::Error::ImproperInclude,
                                    self[pos.range.clone()].to_string(),
                                    filename.to_string_lossy().to_string(),
                                    pos.to_owned()
                                );
                            }
                            pos.range.start.clone()+i
                        };

                        if self[directive_start..pos.range.end].starts_with("include") {
                            directive_start += "include".len();
                        } else {
                            reterr!(
                                super::Error::ImproperInclude,
                                self[pos.range.clone()].to_string(),
                                filename.to_string_lossy().to_string(),
                                pos.to_owned()
                            );
                        }

                        let mut filename_start = {
                            let mut i: usize = 0;
                            for ch 
                            in self[directive_start..pos.range.end].chars()
                            {
                                if ch.is_ascii_whitespace() {
                                    i+=1
                                } else {
                                    break;
                                }
                            }
                            if self[directive_start..pos.range.end]
                                .chars()
                                .try_len()
                                .is_ok_and(|x| x == i)
                            {
                                reterr!(
                                    super::Error::ImproperInclude,
                                    self[pos.range.clone()].to_string(),
                                    filename.to_string_lossy().to_string(),
                                    pos.to_owned()
                                );
                            }
                            directive_start.clone()+i
                        };
                        // SAME STORY HERE
                        /*
                        self[directive_start..pos.range.end]
                            .trim()
                            .find(|x: char| !x.is_ascii_whitespace())
                        else {
                            reterr!(
                                super::Error::ImproperInclude,
                                self[pos.range.clone()].to_string(),
                                filename.to_string_lossy().to_string(),
                                pos.to_owned()
                            );
                        };
                        */

                        if self[filename_start..pos.range.end].starts_with("\"") {

                            self.replace_range(pos.range.clone(), format!("#include \"{}\"", info.filepath).as_str())

                        }
                        
                    }

                    return Ok(());

                }
                Modification::FunctionBody(ty, pos) => {

                    if allowed.contains(&ty) {

                        self.replace_range(pos.range.clone(), ";");
                        //Cleanup space inbetween ')' and ';'
                        let rmwhite = self[..pos.range.start].rfind(|c: char|{!c.is_ascii_whitespace()});
                        if rmwhite.is_some() {
                            self.replace_range((rmwhite.unwrap()+1)..pos.range.start, "");
                        }
                    }

                    return Ok(());

                }
            }
        }
    }

}


// pub(super) mod findings {
    
//     use std::{
//         sync::Mutex,
//         ffi::{c_uint, CStr, CString},
//         ptr::null_mut
//     };
    
//     use lazy_static::lazy_static;
//     use clang_sys::*;

//     #[derive(Debug, Clone)]
//     pub(super) enum Opt {

//         PopAll,
//         Push,
//         PushStart,
//         PushEnd,
//         New

//     }

//     /// Extraction stack to extract (offset start, offset end)
//     /// of a clang_VisitChildren extern "C" function.
//     /// 
//     /// # Warning
//     /// Must be cleaned after or before use with
//     ///  
//     /// The ending offset is not correct as it points to the start of the next
//     /// cursor, to find the end of the function block reverse find a '}' starting
//     /// from end offset.
//     /// ```
//     /// findings::offset_stack(Opt::New, None);
//     /// ```
//     /// 
//     /// # Example
//     /// ```
//     /// // Cleans the stack.
//     /// findings::offset_stack(Opt::New, None);
//     /// 
//     /// // Push a new value:
//     /// findings::offset_stack(Opt::Push, Some((1,1)));
//     /// 
//     /// // Pop all values out of the stack as a vector:
//     /// let vec: Vector<(u32, u32)> = findings::offset_stack(Opt::PopAll, None);
//     /// ```
//     pub(super) fn offset_stack(
//         option: Opt,
//         input: Option<(u32, u32)>
//     ) -> Option<Vec<(u32, u32)>> 
//     { 
//         lazy_static!(
//             static ref OFFSETS: Mutex<Vec<(u32, u32)>> = Mutex::new(vec![]);
//         );
//         match option {
//             Opt::PopAll => {
//                 let tmp = OFFSETS.lock().unwrap().to_vec();
//                 OFFSETS.lock().unwrap().clear();
//                 return Some(tmp);
//             },
//             Opt::Push => {
//                 _ = input.is_some_and(|x| {
//                     OFFSETS.lock().unwrap().push(x); true
//                 });
//             },
//             Opt::New => {
//                 // drop(OFFSETS);
//                 OFFSETS.lock().unwrap().clear();
//             },
//             Opt::PushStart => {
//                 _ = input.is_some_and(|offset_start|{
//                     OFFSETS.lock().unwrap().push((offset_start.0, u32::MAX)); true
//                 });
//             },
//             Opt::PushEnd => {
//                 _ = input.is_some_and(|offset_end|{
//                     let a = OFFSETS.lock().unwrap().pop();
//                     if a.is_some() {
//                         let mut b = a.unwrap().clone();
//                         b.1 = offset_end.1;
//                         OFFSETS.lock().unwrap().push(b);
//                     }
//                     true
//                 });
//             }
//         }
//         None

//     }

//     pub(super) fn inclusion_stack(
//         option: Opt,
//         input: Option<(u32, String)>
//     ) -> Option<Vec<(u32, String)>> 
//     { 
//         lazy_static!(
//             static ref OFFSETS: Mutex<Vec<(u32, String)>> = Mutex::new(vec![]);
//         );
//         match option {
//             Opt::PopAll => {
//                 let tmp = OFFSETS.lock().unwrap().to_vec();
//                 OFFSETS.lock().unwrap().clear();
//                 return Some(tmp);
//             },
//             Opt::Push => {
//                 _ = input.is_some_and(|x| {
//                     OFFSETS.lock().unwrap().push(x); true
//                 });
//             },
//             Opt::New => {
//                 // drop(OFFSETS);
//                 OFFSETS.lock().unwrap().clear();
//             },
//             _ => {}
//         }
//         None

//     }

//     /// Function that gets passed to [`clang_visitChildren`],
//     /// that extracts [`crate::Environment`] ranges into the
//     /// [`crate::offset::stack`] function.
//     /// After [`clang_visitChildren`] is finished you can extract all the
//     /// environment ranges from [`crate::offset::stack`] with 
//     /// `offset::Opt::PushAll`.
//     #[allow(non_snake_case)]
//     pub(super) extern "C" fn from_cursor(

//         ccur: CXCursor,
//         _parent: CXCursor,
//         _data: CXClientData

//     ) -> i32 
//     { unsafe {
        
//         // Filter to find function bodies in current file.
//         if clang_Location_isFromMainFile(
//             clang_getCursorLocation(ccur))
//             == 0 
//         {
            
//             return CXChildVisit_Continue;  
            
//         }
//         else if clang_getCursorKind(ccur) == CXCursor_InclusionDirective
//         {
//             let file: CXFile = clang_getIncludedFile(ccur);
//             let filename: CXString = clang_getFileName(file);
//             eprintln!("# {} #", CStr::from_ptr(filename.data as *mut i8).to_string_lossy().to_string());
//             clang_disposeString(filename);
//             return CXChildVisit_Continue;
//         }
//         else if clang_getCursorKind(ccur) 
//             != CXCursor_CompoundStmt // Compund statment is brackets => { ... }
//         || clang_getCursorKind(clang_getCursorSemanticParent(ccur)) 
//             != CXCursor_FunctionDecl
//         {
            
//             return CXChildVisit_Recurse;
            
//         }
//         else
//         {

//             let range = clang_getCursorExtent(ccur);

//             let mut offset_start: u32 = 0;
//             let mut offset_end: u32 = 0;

//             clang_getFileLocation(
//                 clang_getRangeStart(range), 
//                 null_mut(),
//                 null_mut(),
//                 null_mut(), 
//                 std::ptr::addr_of_mut!(offset_start) as *mut c_uint
//             );

//             // clang_

//             clang_getFileLocation(
//                 clang_getRangeEnd(range), 
//                 null_mut(),
//                 null_mut(),
//                 null_mut(), 
//                 std::ptr::addr_of_mut!(offset_end) as *mut c_uint
//             );

//             offset_stack(Opt::Push, Some((offset_start, offset_end)));

//             return CXChildVisit_Continue;
            
//         }

//     }}

// }

impl Environment {

    /// Get a [`Environment`] from a [`ListerFile`].
    pub fn from_lister( 

        file: &ListerFile,
        cursor: CXCursor
        
    ) -> Result<Environment, Error> 
    { 
        
        let mut full = 
        match std::fs::read_to_string(Path::new(&file.path)) {
            Ok(str) => {
                str
            }
            Err(err) => {
                reterr!(Error::CannotOpenFile,
                    file.path.to_string_lossy().to_string(),
                    err.to_string()
                );
            }
        };

        let mut clean = full.clone();

        let mut modifications: Vec<modify::Modification> = modify::find_from_cursor(cursor);

        for modification in modifications {

            // println!("{:#?}", modification);

            match full.modify(&file.path, &modification, vec![modify::Type::Morph]) {
                Ok(_) => {}
                Err(err) => {return Err(err)}
            }
            match clean.modify(&file.path, &modification, vec![modify::Type::Morph, modify::Type::Remove]) {
                Ok(_) => {}
                Err(err) => {return Err(err)}
            }

        }

        // let mut stack = findings::offset_stack(
        //     findings::Opt::PopAll, 
        //     None
        // ).unwrap();

        // stack.reverse();

        // let mut clean = filestr.clone();
        
        // for offset in stack {

        //     let range = (offset.0 as usize)..(offset.1 as usize);
        //     clean.replace_range(range, ";");
        //     // Cleanup space inbetween ')' and ';'
        //     let pivot: usize = offset.0 as usize;
        //     let rmwhite = clean[..pivot].rfind(|c: char|{!c.is_ascii_whitespace()});
        //     if rmwhite.is_some() {
        //         clean.replace_range((rmwhite.unwrap()+1)..pivot, "");
        //     }

        // }

        Ok(Environment { 
            full:  full.to_owned(),
            clean: clean.to_owned()
        })

    }

}
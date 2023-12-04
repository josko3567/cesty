use crate::{
    error::ErrorPosition, 
    filegroup::FileGroup,
    lister::ListerFile
};

use std::{
    fmt::Display,
    // error::Error,
    collections::HashMap, 
    sync::Mutex,
    ffi::OsString, 
    ptr::null_mut, path::Path,
};

use lazy_static::lazy_static;
use colored::Colorize;
use indoc::formatdoc;
use clang_sys::*;

#[derive(Debug, Clone)]
pub enum Error {

    // FileInPool(ErrorPosition, String),
    CannotOpenFile(ErrorPosition, String, String),

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
        };
        write!(f, "{}\n{message}", 
            format!("From {}...", 
                Path::new(file!())
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap()
                .bold()
            ).dimmed()
        )
    }
}

impl std::error::Error for Error {}

pub struct Environment {
    
    pub full:      String,
    pub bodyclean: String
    
}

// lazy_static!(
//     /// ### pool of file environments.
//     /// 
//     /// Environments can depend on standalone switch,
//     /// therefore we have environment.full which is 
//     /// a copy of the file and environment.bodyclean
//     /// which is a copy of the file without function bodies.
//     #[allow(unused_variables)]
//     #[allow(non_upper_case_globals)]
//     pub static ref ENVIRONMENT_POOL: Mutex<HashMap<OsString, Environment>> = {
//         let a = HashMap::new();
//         Mutex::new(a)
//     };
// );


pub(super) mod offset {
    
    use std::{
        sync::Mutex,
        ffi::c_uint,
        ptr::null_mut
    };
    
    use lazy_static::lazy_static;
    use clang_sys::*;

    #[derive(Debug, Clone)]
    pub(super) enum SOption {

        PopAll,
        Push,
        PushStart,
        PushEnd,
        New

    }

    /// Extraction stack to extract (offset start, offset end)
    /// of a clang_VisitChildren extern "C" function.
    /// 
    /// # Warning
    /// Must be cleaned after or before use with
    ///  
    /// The ending offset is not correct as it points to the start of the next
    /// cursor, to find the end of the function block reverse find a '}' starting
    /// from end offset.
    /// ```
    /// offset_stack(StackOption::New, None);
    /// ```
    /// 
    /// # Example
    /// ```
    /// // Cleans the stack.
    /// offset_stack(StackOption::New, None);
    /// 
    /// // Push a new value:
    /// offset_stack(StackOption::Push, Some((1,1)));
    /// 
    /// // Pop all values out of the stack as a vector:
    /// let vec: Vector<(u32, u32)> = offset_stack(StackOption::PopAll, None);
    /// ```
    pub(super) fn stack(
        option: SOption,
        input: Option<(u32, u32)>
    ) -> Option<Vec<(u32, u32)>> 
    { 
        lazy_static!(
            static ref OFFSETS: Mutex<Vec<(u32, u32)>> = Mutex::new(vec![]);
        );
        match option {
            SOption::PopAll => {
                let tmp = OFFSETS.lock().unwrap().to_vec();
                OFFSETS.lock().unwrap().clear();
                return Some(tmp);
            },
            SOption::Push => {
                _ = input.is_some_and(|x| {
                    OFFSETS.lock().unwrap().push(x); true
                });
            },
            SOption::New => {
                // drop(OFFSETS);
                OFFSETS.lock().unwrap().clear();
            },
            SOption::PushStart => {
                _ = input.is_some_and(|offset_start|{
                    OFFSETS.lock().unwrap().push((offset_start.0, u32::MAX)); true
                });
            },
            SOption::PushEnd => {
                _ = input.is_some_and(|offset_end|{
                    let a = OFFSETS.lock().unwrap().pop();
                    if a.is_some() {
                        let mut b = a.unwrap().clone();
                        b.1 = offset_end.1;
                        OFFSETS.lock().unwrap().push(b);
                    }
                    true
                });
            }
        }
        None

    }

    #[allow(non_snake_case)]
    pub(super) extern "C" fn from_cursor(

        ccur: CXCursor,
        _parent: CXCursor,
        _data: CXClientData

    ) -> i32 
    { unsafe {

        // println!("Hello!");
        // println!("{}",
        //     CStr::from_ptr(clang_getCursorKindSpelling(ccur.kind).data as *const i8).to_string_lossy().to_string()

        // );
        
        // Filter to find function bodies in current file.
        if clang_Location_isFromMainFile(
            clang_getCursorLocation(ccur))
            == 0 
        {
            
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
        // println!("Hello!");

            let range = clang_getCursorExtent(ccur);

            let mut offset_start: u32 = 0;
            let mut offset_end: u32 = 0;

            clang_getFileLocation(
                clang_getRangeStart(range), 
                null_mut(),
                null_mut(),
                null_mut(), 
                std::ptr::addr_of_mut!(offset_start) as *mut c_uint
            );

            clang_getFileLocation(
                clang_getRangeEnd(range), 
                null_mut(),
                null_mut(),
                null_mut(), 
                std::ptr::addr_of_mut!(offset_end) as *mut c_uint
            );

            stack(SOption::Push, Some((offset_start, offset_end)));

            return CXChildVisit_Continue;
            
        }

    }}

}

impl Environment {

    pub fn from_lister_into_pool( 

        file: &ListerFile,
        cur: CXCursor
        
    ) -> Result<Environment, Error> 
    { 

        // if ENVIRONMENT_POOL.lock().unwrap().contains_key(&file.path) {
        //     reterr!(Error::FileInPool,
        //         file.path.clone().to_string_lossy().to_string()
        //     );
        // }
        
        let filestr = 
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
    
        offset::stack(offset::SOption::New, None);
            
        unsafe { clang_visitChildren(
            cur,
            offset::from_cursor,
            null_mut()
        )};

        let mut stack = offset::stack(
            offset::SOption::PopAll, 
            None
        ).unwrap();

        stack.reverse();

        let mut clean = filestr.clone();
        
        for offset in stack {

            let range = (offset.0 as usize)..(offset.1 as usize);
            clean.replace_range(range, ";");

        }

        Ok(Environment { 
            full: filestr.to_owned(),
            bodyclean: clean.to_owned()
        })

        // println!("{}", clean);
        // stack.iter().for_each(|x| println!("{:?}", x));

        // Ok(())

        


    }

}
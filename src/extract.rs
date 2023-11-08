use crate::lister::ListerFile;

use std::{
    ffi::{CStr, c_uint, OsString},
    ptr::null_mut,
};

use clang_sys::*;


#[derive(Debug, Clone)]
enum EOptions {

    PopAll,
    Push,
    New
    
}

#[derive(Clone, Default)]
pub struct ExtractTest {

    pub comment:  Vec<String>,
    pub function: String,
    pub returns:  String,
    pub line:     u32,
    pub column:   u32

}

#[derive(Clone, Default)]
pub struct Extract {

    pub filepath: OsString,
    pub tests:    Vec<ExtractTest>

}

fn extract_stack(
    option: EOptions,
    input: Option<ExtractTest>
) -> Option<Vec<ExtractTest>> 
{ unsafe {

    static mut TESTS: Vec<ExtractTest> = vec![];
    match option {
        EOptions::PopAll => {
            let tmp = TESTS.to_vec();
            TESTS = vec![];
            return Some(tmp);
        },
        EOptions::Push => {
            _ = input.is_some_and(|x| {
                TESTS.push(x); true
            });
        },
        EOptions::New => {
            TESTS = vec![];
        }
    }
    None

}}

#[repr(u8)]
enum CommentType {

    Multi,
    TripleDash

}

fn parse_comment(comment: String) -> Option<Vec<String>> {

    let lines: Vec<&str> = comment
        .split_once("#!cesty;")?
        .1
        .split('\n')
        .filter(|x| 
            x
            .chars()
            .find(|y| !y.is_ascii_whitespace())
            .is_some()
        ).collect();

    let mut comment:  String      = String::new();
    let mut comments: Vec<String> = vec![];
    let mut skip:     usize       = 0;
    let mut inyaml:   bool        = false;

    for line in lines {

        if inyaml == false // at the start ---
        {

            // Find the position in front of the
            // last character
            // of the last word behind "---"
            // Example:
            // * --- => 1
            // *** --- => 3
            //  *** --- => 4
            //  ***--- => 4
            // --- => 0
            // Used for skipping multiline comments that
            // start with * in each line.
            skip = match line.split_once("---") 
            {
                Some(slices) => 
                {
                    match slices
                        .0
                        .rfind(|x: char| !x.is_whitespace()) 
                    {
                        Some(pos) => {pos+1}
                        None => {0}
                    }
                }
                None =>
                {
                    break;
                }
            };
            inyaml = true;

        }
        else if  // At the end of a yaml ...
            line
                .split_at(skip)
                .1
                .trim_start()
                .starts_with("...")
        &&  inyaml == true
        {
            comments.push(comment);
            comment = String::new();

            inyaml = false;
            skip = 0;   
        }
        else 
        {
            comment.push_str(
                line
                    .split_at(skip)
                    .1
                    .trim_end()
            );
            comment.push('\n');
        }

        // println!("{line}");

    }

    Some(comments)

}

#[allow(non_snake_case)]
extern "C" fn extract_from_cursor(

    ccur: CXCursor,
    _parent: CXCursor,
    _data: CXClientData

) -> i32 
{ unsafe {

    if clang_Location_isFromMainFile(
        clang_getCursorLocation(ccur)) == 0 
    || ccur.kind != CXCursor_FunctionDecl {

        return CXChildVisit_Continue;  

    }
    
    let ccur_comment = clang_Cursor_getRawCommentText(
        ccur
    );

    if ccur_comment.data.is_null() {
        return CXChildVisit_Continue;
    }

    let ccur_dname = clang_getCursorDisplayName(
        ccur
    );

    
    
    let ccur_tyspell = clang_getTypeSpelling(
        clang_getResultType(
            clang_getCursorType(ccur)
        )
    );

    if ccur_dname.data.is_null() 
    || ccur_tyspell.data.is_null() {
        
        clang_disposeString(ccur_comment);
        clang_disposeString(ccur_dname);
        clang_disposeString(ccur_tyspell);
        return CXChildVisit_Continue;
        
    }

    let mut ccur_line: u32 = 0;
    let mut ccur_col: u32 = 0;
    
    let cpos = clang_getCursorLocation(ccur);
    clang_getExpansionLocation(
        cpos,
        null_mut(),
        std::ptr::addr_of_mut!(ccur_line) as *mut c_uint, 
        std::ptr::addr_of_mut!(ccur_col) as *mut c_uint,
        null_mut()
    );

    let ccur_comment_str = 
        CStr::from_ptr(ccur_comment.data as *const i8)
        .to_string_lossy()
        .to_string();
    
    let ccur_dname_str = 
        CStr::from_ptr(ccur_dname.data as *const i8)
        .to_string_lossy()
        .to_string();
    
    let ccur_tyspell_str = 
        CStr::from_ptr(ccur_tyspell.data as *const i8)
        .to_string_lossy()
        .to_string();

    clang_disposeString(ccur_comment);
    clang_disposeString(ccur_dname);
    clang_disposeString(ccur_tyspell);

    let parsed_comments = 
    match parse_comment(ccur_comment_str) {
        Some(ret) => {ret},
        _ => {return CXChildVisit_Continue;}
    };

    extract_stack(
        EOptions::Push, 
        Some(ExtractTest {
            comment:  parsed_comments, 
            function: ccur_dname_str, 
            returns:  ccur_tyspell_str,
            line:     ccur_line, 
            column:   ccur_col
        }
    ));

    return CXChildVisit_Continue;  

}}

impl Extract {
    
    pub fn from_lister(

        file: &ListerFile,
        cur: CXCursor
        
    ) -> Extract 
    { 
    
        extract_stack(EOptions::New, None);

        unsafe{ clang_visitChildren(
            cur,
            extract_from_cursor,
            null_mut()
        );}

        Extract {
            filepath: file.path.clone(),
            tests: match extract_stack(
                EOptions::PopAll, None) {
                    Some(ext) => {ext},
                    _ => {vec![]}
                }
        }
    
    }

    
}
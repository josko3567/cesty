use crate::{
    lister::ListerFile,
    error::ErrorGroup
};

use std::{
    ffi::{CString, CStr, c_uint, OsString, c_void},
    ptr::{null, null_mut},
    error::Error,
    fmt::Display, path::Path
};

use clang_sys::*;
use indoc::indoc;
use colored::Colorize;

#[derive(Debug, Clone)]
pub enum ExtractError {

    NothingToExtract(String),
    IndexInitFailed,
    TranslationUnitInitFailed(String),

}

impl ExtractError {

    pub fn code(&self) -> String {
        return format!("E;{:X}:{:X}", 
            ErrorGroup::from(
                Path::new(file!())
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap()
            ) as u8, 
            unsafe { *(self as *const Self as *const u8) }
        );
    }
    
}

impl Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::NothingToExtract(str) => {
                format!( indoc!{"
                Error! 
                    Nothing was extracted from file:
                        {}
                "}, str.underline())
                .red()},
            Self::IndexInitFailed => {
                format!( indoc!{"
                Error! 
                    clang_createIndex() returned NULL, aka. clang failed
                    to initialize!.
                "})
                .red()},
            Self::TranslationUnitInitFailed(str )=> {
                format!( indoc!{"
                Error! 
                    clang_parseTranslationUnits() returned NULL, aka. 
                    clang failed to translate the file, probably
                    because:
                        {}
                    Does not contain valid .c
                "}, str.bold().red())
                .red()}
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

impl Error for ExtractError {}

#[derive(Debug, Clone)]
enum ExtExtionOptions {

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
    option: ExtExtionOptions,
    input: Option<ExtractTest>
) -> Option<Vec<ExtractTest>> 
{ unsafe {

    static mut TESTS: Vec<ExtractTest> = vec![];
    match option {
        ExtExtionOptions::PopAll => {
            let tmp = TESTS.to_vec();
            TESTS = vec![];
            return Some(tmp);
        },
        ExtExtionOptions::Push => {
            _ = input.is_some_and(|x| {
                TESTS.push(x); true
            });
        },
        ExtExtionOptions::New => {
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
        ExtExtionOptions::Push, 
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
        
    ) -> Result<Extract, ExtractError> 
    { unsafe{
    
        extract_stack(ExtExtionOptions::New, None);

        // let cur = clang_getTranslationUnitCursor(tu);
        
        clang_visitChildren(
            cur,
            extract_from_cursor,
            null_mut()
        );

        Ok(Extract {
            filepath: file.path.clone(),
            tests: match extract_stack(
                ExtExtionOptions::PopAll, None) {
                    Some(ext) => {ext},
                    _ => {vec![]}
                }
        })
    
    }}

    
}
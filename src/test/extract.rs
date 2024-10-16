//! Functions for extracting test data from a file.
//! 
//! It's quite messy code, can't really help it unlike
//! the rest of the codebase.
//! 
//! Main function is [extract] for extracting a [ParsedFile]
//! from a [PathBuf].
//! 
//! The module [visitor] is completely private and contains
//! the filter for [clang_sys::clang_visitChildren] & other
//! functions used inside of it.

use std::{ffi::OsString, path::PathBuf};

use crate::{
    error::{
        debuginfo, debugpush, error, 
        Alert, AlertInfo
    }, 
    rustclang::{
        Clang, Open
    }
};


/// Function name & return type.
#[derive(Clone, Debug, Default)]
pub struct Function {

    /// Return type in string. Might be removed due to
    /// redundancy.
    pub returns: String,

    /// Full function name (with *cesty_*)
    pub name: String,

    /// Part of the full name (without the *cesty_*)
    /// 
    /// For normal function names like *main* this is empty.
    pub name_slice: String,

    /// Arguments that the function accepts. 
    pub args: Vec<String>

}

/// Ranges for certain parts of a function.
#[derive(Clone, Debug, Default)]
pub struct Range {
    
    /// Range of the function template.
    pub template: (usize, usize),

    /// Range of the function body (from **{** to **}**).
    pub body: (usize, usize)

}

/// Everything useful that is extracted from a test function.
#[derive(Clone, Debug, Default)]
pub struct ParsedTest {

    /// Parsed from the function docs.
    pub config:   super::Config,

    /// Function template data.
    pub function: Function,

    /// Ranges of certain function parts.
    pub range:    Range

}

/// Contains the environment the test will be placed in.
#[derive(Clone, Debug, Default)]
pub struct Environment {

    /// Full file.
    pub full:      String,

    /// Full file without main().
    pub mainless:  String,

    /// File without all the function bodies & main().
    pub templated: String

}

/// Parsed file test data.
#[derive(Clone, Debug, Default)]
pub struct ParsedFile {

    /// File path with filename.
    pub path: PathBuf,

    /// File stem.
    pub stem: OsString,

    /// List of all tests found inside of the file.
    pub test: Vec<ParsedTest>,

    /// The files full & clean environment.
    pub environment: Environment,

    /// Main function.
    pub main: Option<ParsedTest>

}

impl ParsedTest {

    /// Test file stem used for debuging.
    #[allow(dead_code)]
    pub fn get_test_file_stem(
        &self, 
        parsed_file: &ParsedFile
    ) -> OsString {

        let mut stem = parsed_file.stem.clone();
        stem.push(OsString::from("_").as_os_str());
        stem.push(self.function.name_slice.clone());
        stem

    }

}

pub fn extract(
    path: PathBuf
) -> Result<(ParsedFile, Vec<Alert>), Alert> {

    let mut warnings: Vec<Alert> = vec![];

    let clang = match Clang::open(&path, "-fparse-all-comments") {
        Ok(clang) => clang,
        Err(err) => { return Err(debugpush!(err)) }
    };

    match visitor::visit(path.as_path(), &clang) {
        Ok(((tests, main, environment), mut ext_warnings)) => {
            warnings.append(&mut ext_warnings);
            return Ok((ParsedFile {
                path: path.clone(),
                stem: if path.file_stem().is_some() {
                    path.file_stem().unwrap().to_os_string()
                } else {
                    return error!{
                        description: "failed to extract file stem from path".to_owned(),
                        example: None,
                        debug: debuginfo!(),
                        note: vec!["path.file_stem() returned `None`".to_owned()]
                    }
                },
                test: tests,
                environment,
                main
            }, warnings))
        }
        Err(err) => {
            return Err(debugpush!(err));
        }
    };

}

pub mod visitor {

    use std::path::Path;
    // use std::str::pattern::Pattern;
    use std::{
        cell::RefCell, 
        ptr::null_mut
    };
    use clang_sys as libclang;
    use strum::{EnumProperty, IntoEnumIterator};
    use crate::{
        rustclang::{
            Clang, self, 
            cxstring_to_string_consumable, 
            filename_from_cursor 
        },
        defaults::DEFAULT_FUNCTION_PREFIX, 
        error::{
            debugappend, debuginfo, 
            debugpush, error, 
            warning, Alert, AlertCode, 
            AlertCodeFix, AlertExample, 
            AlertInfo, function_message
        },
        test::Config
    };

    use super::{Environment, ParsedTest};

    #[derive(
        strum_macros::EnumProperty, 
        strum_macros::EnumIter,
        strum_macros::Display,
        Debug
    )]
    enum CommentVariant {

        #[strum(props(mark = "//"))]
        Singleline,
        #[strum(props(open = "/*", close = "*/", between = "*"))]
        Multiline

    }

    impl CommentVariant {


        /// Since comments returned by [libclang::clang_Cursor_getRawCommentText]
        /// can have multiple comment variants, this function
        /// separates them into a Vec<(String, CommentVariant)>.
        /// 
        /// Returns
        /// -------
        /// If the function did not contain any documentation
        /// comment, we return a [Ok()] containing a [None].
        /// 
        /// If the function had valid documentation comments,
        /// we return a [Ok()] containing a [Some] of [Vec<(String, Self)>]
        /// 
        /// If something went disarray, we return a [Err()]
        /// containing a [Alert]. 
        /// 
        /// Issues
        /// ------
        /// One issue i found while testing is that [libclang]
        /// messes up if a **singleline** comment starts right after 
        /// the end delimiter of a **multiline** comment:
        /// 
        /// ```C
        /// /*
        ///  Multiline
        ///  */ // OH NO SINGLELINE
        /// ```
        /// 
        /// Probably a faulty parser but [libclang::clang_Cursor_getRawCommentText]
        /// will return NULL for the example shown.
        /// 
        /// Im not going to write a C comment parser just
        /// becase for some god forsaken reason you might
        /// write comments like this.
        /// 
        /// If you do, don't, thanks in advance.
        fn comment_slices_from_cursor(
            cursor: libclang::CXCursor
        ) -> Result<Option<Vec<(String, Self)>>, Alert> {

            if unsafe {libclang::clang_getCursorKind(cursor)} != libclang::CXCursor_FunctionDecl {
                return error!{
                    description: "invalid CXCursor::kind as argument".to_owned(),
                    debug: debuginfo!(),
                    example: None,
                    note: vec![format!("expected a `CXCursor_FunctionDecl` as the CXCursor::kind, received `{}`.", 
                        rustclang::cxstring_to_string_consumable(
                            unsafe {libclang::clang_getCursorKindSpelling(libclang::clang_getCursorKind(cursor))}
                        ).unwrap_or_else(|_|{"unknown".to_owned()})
                    )]
                };
            }
            
            let mut comment = match cxstring_to_string_consumable(
                unsafe { libclang::clang_Cursor_getRawCommentText(cursor) }
            ) {

                Ok(comment) => comment,
                Err(_) => return Ok(None)

            }.trim().to_owned();

            let mut comment_slices = vec![];

            while comment.trim().is_empty() == false {

                let current_slices = comment_slices.len();

                for variant in Self::iter() {
    
                    let Some(mark) = variant
                        .get_str("mark").or(variant
                        .get_str("open")) 
                    else {
                        return error!{
                            debug: debuginfo!(),
                            description: "missing `mark` or `open` property for CommentVariant".to_string(),
                            example: None,
                            note: vec![
                                format!("CommentVariant::{} does not have an `mark` or a `open` property", variant),
                                "this is a developer issue, please publish this issue on the github page".to_owned(),
                            ]
                        }
                    };
    
                    if comment.starts_with(mark) {

                        match variant {

                            Self::Multiline => {

                                let Some(closing_delimiter) = variant.get_str("close") 
                                else {
                                    return error!{
                                        debug: debuginfo!(),
                                        description: "missing `close` property for CommentVariant".to_string(),
                                        example: None,
                                        note: vec![
                                            format!("CommentVariant::{} does not have an `close` property", variant),
                                            "this is a developer issue, please publish this issue on the github page".to_owned(),
                                        ]
                                    }
                                };

                                let position = match comment.find(closing_delimiter) {
                                    Some(position) => position,
                                    None => {
                                        let path = rustclang::filename_from_cursor(cursor).unwrap_or("".to_owned());

                                        let (code, line, column) = (match std::fs::read_to_string(&path) {
                                            Ok(code) => code,
                                            Err(_) => "{unknown}".to_owned()
                                        }, rustclang::position_from_cursor_extent(cursor).0.0, rustclang::position_from_cursor_location(cursor).0.1);

                                        return error!{
                                            debug: debuginfo!(),
                                            description: format!("multiline comment is missing closing delimiter `{closing_delimiter}`"),
                                            example: Some(AlertExample::Code(AlertCode{
                                                code: code,
                                                fix: vec![
                                                    AlertCodeFix {
                                                        relative_line: line - 1,
                                                        column: column,
                                                        comment: format!(
                                                            "this function has a multiline comment above it that doesn't have a closing `{closing_delimiter}` delimiter"
                                                        )
                                                    }
                                                ],
                                                line: line,
                                                file: path
                                            })),
                                            note: vec![]
                                        }
                                    }
                                };

                                let splits = comment.split_at_mut(position + closing_delimiter.len());
                                comment_slices.push( (splits.0.to_owned(), variant) );
                                comment = splits.1.trim().to_owned();
                                break;
                                
                            },
                            
                            Self::Singleline => {

                                let mut accumulation: Vec<String> = vec![];


                                loop {

                                    accumulation.push(String::new());

                                    if comment.trim_start().starts_with(mark) {

                                        for ch in comment.chars() {

                                            accumulation.last_mut().unwrap().push(ch);

                                            if ch == '\n' { break }
                                            
                                        }

                                        _ = comment.drain(..accumulation.last().unwrap().len());

                                    }

                                    if accumulation.last().unwrap().trim().is_empty() {
                                        break;
                                    }

                                }

                                comment_slices.push((accumulation.join(""), variant));

                                break;

                            }

                        }

                    }
    
                }

                if current_slices == comment_slices.len() {

                    let path = rustclang::filename_from_cursor(cursor).unwrap_or("".to_owned());

                    let (code, line, column) = (match std::fs::read_to_string(&path) {
                        Ok(code) => code,
                        Err(_) => "{unknown}".to_owned()
                    }, rustclang::position_from_cursor_extent(cursor).0.0, rustclang::position_from_cursor_location(cursor).0.1);

                    let mut available_types = {

                        let mut variants: Vec<String> = vec![];

                        for variant in Self::iter() {

                            let mut s = format!("CommentVariant::{}", variant.to_string());

                            match variant.get_str("mark") {
                                Some(mark) =>  s = s + format!(" with mark `{}`", mark).as_str(),
                                None => ()
                            }

                            match variant.get_str("open") {
                                Some(open) =>  {
                                    match variant.get_str("close") {
                                        Some(close) => s = s + format!(" with opening delimiter `{}` and closing delimiter `{}`", open, close).as_str(),
                                        None => ()
                                    }
                                }
                                None => ()
                            }

                            variants.push(s);

                        }

                        variants

                    };

                    let mut note = vec![
                        "supported comment variants are:".to_owned()
                    ];

                    note.append(&mut available_types);

                    return error!{
                        debug: debuginfo!(),
                        description: format!("parsing multiple comment variants failed"),
                        example: Some(AlertExample::Code(AlertCode{
                            code: code,
                            fix: vec![
                                AlertCodeFix {
                                    relative_line: line - 1,
                                    column: column,
                                    comment: format!(
                                        "this function has a unknown comment variant that was not detected."
                                    )
                                }
                            ],
                            line: line,
                            file: path
                        })),
                        note: note
                    }

                }


            }

            Ok(Some(comment_slices))

        }

    }

    /// Cleans up the raw config comment.
    /// 
    /// Example of comments
    /// -------
    /// ```C
    /// /**
    ///  * Hello world!
    ///  * Lorem
    ///  * Ipsum 
    ///  */
    /// 
    /// // Hello world!
    /// // Lorem
    /// // Ipsum
    /// 
    /// /*Hello world!
    /// Lorem
    /// Ipsum*/
    /// 
    /// ///// Hello world!
    /// // Lorem
    /// //////// Ipsum
    /// 
    /// /**
    ///  * Hello world! 
    /// Lorem
    /// */ 
    /// ///// Ipsum
    /// ```
    /// 
    /// From all of these the original text must be extracted:
    /// ```None
    /// Hello world!
    /// Lorem
    /// Ipsum
    /// ```
    /// 
    /// Heres a bit more of a cursed example:
    /// ```C
    /// /*****[compiler]*/
    /// /**name = "clang"
    ///  * [[test]]
    ///    name   = "Pass."
    /// input  = {definition = "BOOK_SALES"}
    /// ****output = true*/
    /// //[[test]]
    /// //name   = "Fail."
    /// //////  desc   = "Purposefull fail."
    /// //////////input  = 1
    /// //////  output = false
    /// //
    /// //
    /// /** 
    ///  * 
    ///  * 
    ///  * 
    ///  * 
    ///  * 
    ///  * 
    /// */
    /// //
    /// /*****[[test]]
    /// */
    /// /////// name = "Cum." 
    /// ///
    /// ```
    /// 
    /// From this we extract:
    /// ```toml
    /// [compiler]
    /// name = "clang"
    /// [[test]]
    /// name   = "Pass."
    /// input  = {definition = "BOOK_SALES"}
    /// output = true
    /// [[test]]
    /// name   = "Fail."
    /// desc   = "Purposefull fail."
    /// input  = 1
    /// output = false
    /// [[test]]
    /// name = "Cum."
    /// ```
    fn config_string_from_cursor(
        cursor: libclang::CXCursor
    ) -> Result<Option<Vec<(String, String, usize, usize)>>, Alert> {

        let (total_lines, comments) = match CommentVariant::comment_slices_from_cursor(cursor) {
            Ok(schrodingers_comments) => match schrodingers_comments {
                Some(parsed_comments) => 
                    (
                        parsed_comments.iter()
                                       .fold(0, |acc, x: &(String, CommentVariant)| 
                                            {acc + x.0.lines().count()})
                        ,parsed_comments
                    ),
                None => return Ok(None) 
            }
            Err(err) => return Err(debugpush!(err))
        };

        let start_line = rustclang::position_from_cursor_location(cursor).0.0 - total_lines;
        let filename = rustclang::filename_from_cursor(cursor).unwrap_or("unknown".to_owned());
        
        let mut accumulated_lines = 0;
        let mut accumulation: Vec<(String, String, usize, usize)> = vec![]; 

        for (comment_string, comment_variant) in comments {

            match comment_variant {
                
                CommentVariant::Singleline => {

                    let comment_lines = comment_string.lines();

                    let Some(mark) = comment_variant.get_str("mark") 
                    else {
                        return error!{
                            debug: debuginfo!(),
                            description: "missing `mark` property for CommentVariant".to_string(),
                            example: None,
                            note: vec![
                                format!("CommentVariant::{} does not have an `mark` property", comment_variant),
                                "this is a developer issue, please publish this issue on the github page".to_owned(),
                            ]
                        }
                    };


                    for line in comment_lines {

                        let trimmed = line.trim_start();

                        if trimmed.starts_with(mark) {

                            match line.find(|c: char| 
                                !mark.contains(c) && !c.is_whitespace()
                            ) {

                                Some(position) => {
                                    let extracted_comment_line = trimmed.split_at(position).1.to_owned();
                                    accumulation.push((extracted_comment_line, line.to_owned(), accumulated_lines+start_line, position));
                                    accumulated_lines += 1;
                                },
                                None => {
                                    if trimmed
                                        .split_at(mark.len())
                                        .1
                                        .trim()
                                        .find(|c: char| !mark.contains(c) && !c.is_whitespace())
                                        .is_none() 
                                    {

                                        accumulated_lines += 1;
                                        continue;

                                    } else {
                                        return error!{
                                            debug: debuginfo!(),
                                            description: "unable to locate starting position for comment part".to_owned(),
                                            example: Some(AlertExample::Code(AlertCode{
                                                code: line.to_string(),
                                                fix: vec![
                                                    AlertCodeFix {
                                                        relative_line: 0,
                                                        column: line.find(|c: char| {!c.is_whitespace()}).unwrap_or(0),
                                                        comment: format!(
                                                            "this comment part failed to be parsed into a config part"
                                                        )
                                                    }
                                                ],
                                                line: start_line + accumulated_lines,
                                                file: filename
                                            })),
                                            note: vec![]
                                        }
                                    }
                                }

                            };

                        } else {

                            return error!{
                                debug: debuginfo!(),
                                description: "invalid comment".to_owned(),
                                example: Some(AlertExample::Code(AlertCode{
                                    code: line.to_string(),
                                    fix: vec![
                                        AlertCodeFix {
                                            relative_line: 0,
                                            column: line.find(|c: char| {!c.is_whitespace()}).unwrap_or(0),
                                            comment: format!(
                                                "expected a `{}` at the start", mark
                                            )
                                        }
                                    ],
                                    line: start_line + accumulated_lines,
                                    file: filename
                                })),
                                note: vec![
                                    format!("Expected a line to start with {mark} for CommentVariant::{}", comment_variant.to_string())
                                ]
                            }

                        }

                    }

                }
                CommentVariant::Multiline => {

                    let Some(opening_delimiter) = comment_variant.get_str("open").or(comment_variant.get_str("mark"))
                    else {
                        return error!{
                            debug: debuginfo!(),
                            description: "missing `mark` or `open` property for CommentVariant".to_string(),
                            example: None,
                            note: vec![
                                format!("CommentVariant::{} does not have an `mark` or a `open` property", comment_variant),
                                "this is a developer issue, please publish this issue on the github page".to_owned(),
                            ]
                        }
                    };

                    let Some(closing_delimiter) = comment_variant.get_str("close") 
                    else {
                        return error!{
                            debug: debuginfo!(),
                            description: "missing `close` property for CommentVariant".to_string(),
                            example: None,
                            note: vec![
                                format!("CommentVariant::{} does not have an `close` property", comment_variant),
                                "this is a developer issue, please publish this issue on the github page".to_owned(),
                            ]
                        }
                    };

                    let Some(between) = comment_variant.get_str("between") 
                    else {
                        return error!{
                            debug: debuginfo!(),
                            description: "missing `between` property for CommentVariant".to_string(),
                            example: None,
                            note: vec![
                                format!("CommentVariant::{} does not have an `between` property", comment_variant),
                                "this is a developer issue, please publish this issue on the github page".to_owned(),
                            ]
                        }
                    };

                    let comment_line_delimited = {

                        let comment_trimmed = comment_string.trim();
                        let comment_start_delimited = if comment_trimmed.starts_with(opening_delimiter) {
                            
                            comment_trimmed.split_at(opening_delimiter.len()).1
                        
                        } else {

                            unreachable!();

                        };

                        let comment_start_end_delimited = if comment_start_delimited.ends_with(closing_delimiter) {   

                            comment_start_delimited.split_at(comment_start_delimited.len() - closing_delimiter.len()).0
                        
                        } else {

                            unreachable!()

                        };

                        comment_start_end_delimited

                    }.lines();

                    for line in comment_line_delimited {

                        let trimmed = line.trim_start();

                        match line.find(|c: char|
                            !between.contains(c) && !c.is_whitespace()
                        ) {
                            Some(position) => {
                                let extracted_comment_line = trimmed.split_at(position).1.to_owned();
                                accumulation.push((extracted_comment_line, line.to_owned(), accumulated_lines+start_line, position));
                                accumulated_lines += 1;
                            },
                            None => {
                                if trimmed
                                    .trim_end()
                                    .find(|c: char| !between.contains(c) && !c.is_whitespace())
                                    .is_none() 
                                {

                                    accumulated_lines += 1;
                                    continue;

                                } else {

                                    return error!{
                                        debug: debuginfo!(),
                                        description: "unable to locate starting position for comment part".to_owned(),
                                        example: Some(AlertExample::Code(AlertCode{
                                            code: line.to_string(),
                                            fix: vec![
                                                AlertCodeFix {
                                                    relative_line: 0,
                                                    column: line.find(|c: char| {!c.is_whitespace()}).unwrap_or(0),
                                                    comment: format!(
                                                        "this comment part failed to be parsed into a config part"
                                                    )
                                                }
                                            ],
                                            line: start_line + accumulated_lines,
                                            file: filename
                                        })),
                                        note: vec![]
                                    }

                                }

                            }
                            
                        };

                    }

                }

            }

        }

        Ok(Some(accumulation))

    }

    fn valid_function_name_from_cursor(
        cursor: libclang::CXCursor
    ) -> Result<(Option<(String, String)>, Vec<Alert>), Alert> { unsafe {

        let mut warnings: Vec<Alert> = vec![];

        if libclang::clang_getCursorKind(cursor) != libclang::CXCursor_FunctionDecl {
            return error!{
                description: "invalid CXCursor::kind as argument".to_owned(),
                debug: debuginfo!(),
                example: None,
                note: vec![format!("expected a `CXCursor_FunctionDecl` as the CXCursor::kind, received `{}`.", 
                    rustclang::cxstring_to_string_consumable(
                        libclang::clang_getCursorKindSpelling(libclang::clang_getCursorKind(cursor))
                    ).unwrap_or_else(|_|{"unknown".to_owned()})
                )]
            };
        }

        let function_name = match rustclang::cxstring_to_string_consumable(
            libclang::clang_getCursorSpelling(cursor)
        ) {
            Ok(s) => s.trim().to_owned(),
            Err(err) => return Err(debugpush!(err))
        };

        if function_name.starts_with(DEFAULT_FUNCTION_PREFIX) == true {

            let test_name_part = function_name[DEFAULT_FUNCTION_PREFIX.len()..].to_owned();

            if test_name_part.is_empty() {

                let (code, relative_line, column) = match rustclang::filename_from_cursor(cursor) {
                    Ok(filename) => {
                        match std::fs::read_to_string(filename) {
                            Ok(code) => 
                                (code, rustclang::position_from_cursor_extent(cursor).0.0 - 1, rustclang::position_from_cursor_location(cursor).0.1 + function_name.len()),
                            Err(_) => (function_name.clone(), 0 as usize, function_name.len() + 1)
                        }
                    }
                    Err(_) => (function_name.clone(), 0 as usize, function_name.len() + 1)
                };

                warnings.push(warning!{
                    description: format!("function only contains prefix part aka. `{}`", function_name),
                    debug: debuginfo!(),
                    example: Some(AlertExample::Code(AlertCode{
                        code: code,
                        fix: vec![
                            AlertCodeFix {
                                relative_line: relative_line,
                                column: column,
                                comment: "add a name for the test like `sum_test` or anything you like`".to_owned()
                            }
                        ],
                        line: rustclang::position_from_cursor_location(cursor).0.0 as usize,
                        file: rustclang::filename_from_cursor(cursor).unwrap_or("unknown".to_owned())
                    })),
                    note: vec![
                        format!("due to having no name the test will be ignored")
                    ]
                });

                return Ok((None, warnings));

            }

            Ok((Some((function_name, test_name_part)), warnings))

        } else {

            Ok((None, warnings))

        }


    }}

    // This was the only way to use thread local mut statics,
    // i guess rust doesn't like this to much since rust-analyzer
    // (vscode :P) doesn't really know what to suggest when 
    // typing like. TEST_STACK.with(...), it just dies ;-;
    thread_local!{

        static TEST_STACK:  RefCell<Vec<ParsedTest>>                    = RefCell::new(vec![]);
        //                               Start, end,  is main
        static CLEAN_STACK: RefCell<Vec<(usize, usize, bool)>>          = RefCell::new(vec![]);
        static WARNINGS:    RefCell<Vec<Alert>>                         = RefCell::new(vec![]);
        //                              mane info, start, end
        static MAIN:        RefCell<Option<(ParsedTest, usize, usize)>> = RefCell::new(None);
        static ERROR:       RefCell<Option<Alert>>                      = RefCell::new(None);

    }

    pub fn visit(
        path: &Path,
        clang: &Clang
    ) -> Result<((Vec<ParsedTest>, Option<ParsedTest>, Environment), Vec<Alert>), Alert> {

        TEST_STACK .with(|n| n.borrow_mut().clear());
        CLEAN_STACK.with(|n| n.borrow_mut().clear());
        WARNINGS   .with(|n| n.borrow_mut().clear());
        MAIN       .with(|n| (*n.borrow_mut()) = None);
        ERROR      .with(|n| (*n.borrow_mut()) = None);
        
        extern "C" fn filter(

            cursor: libclang::CXCursor,
            _p:     libclang::CXCursor,
            _d:     libclang::CXClientData
    
        ) -> libclang::CXChildVisitResult { unsafe {


            if libclang::clang_Location_isFromMainFile(
                libclang::clang_getCursorLocation(cursor)
                ) == 0
            {
                return libclang::CXChildVisit_Continue;
            }
            else if libclang::clang_getCursorKind(cursor) == libclang::CXCursor_FunctionDecl
            {
                return libclang::CXChildVisit_Recurse;   
            }
            else if libclang::clang_getCursorKind(libclang::clang_getCursorSemanticParent(cursor)) == libclang::CXCursor_FunctionDecl
                 && libclang::clang_getCursorKind(cursor) != libclang::CXCursor_CompoundStmt
            {
                return libclang::CXChildVisit_Continue;
            }
            else if libclang::clang_getCursorKind(libclang::clang_getCursorSemanticParent(cursor)) == libclang::CXCursor_FunctionDecl
                 && libclang::clang_getCursorKind(cursor) == libclang::CXCursor_CompoundStmt
            {

                let body_range = rustclang::range_from_cursor_extent(cursor);

                let template_range = (
                    rustclang::range_from_cursor_extent(libclang::clang_getCursorSemanticParent(cursor)).0,
                    rustclang::range_from_cursor_extent(cursor).0 - 1,
                );

                let function_name = match rustclang::cxstring_to_string_consumable(
                    libclang::clang_getCursorSpelling(libclang::clang_getCursorSemanticParent(cursor))
                ) {
                    Ok(s) => s.trim().to_owned(),
                    Err(err) => {
                        ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                        return libclang::CXChildVisit_Break;
                    }
                }; 

                // Special treatment for main.
                if function_name == "main" {
                    CLEAN_STACK.with(|n| n.borrow_mut().push((template_range.0, body_range.1, true)))
                } else {
                    CLEAN_STACK.with(|n| n.borrow_mut().push((body_range.0, body_range.1, false)))
                }

                // Get qualified function names only, special treatment for main.
                let (full_function_name, full_cesty_name) = match valid_function_name_from_cursor(
                    libclang::clang_getCursorSemanticParent(cursor)
                ) {
                    Ok((result, warnings)) => {
                        WARNINGS.with(|n| n.borrow_mut().append(&mut debugappend!(warnings)));
                        match result {
                            Some(valid) => valid,
                            None => {
                                let function_name = match rustclang::cxstring_to_string_consumable(
                                    libclang::clang_getCursorSpelling(libclang::clang_getCursorSemanticParent(cursor))
                                ) {
                                    Ok(s) => s.trim().to_owned(),
                                    Err(err) => {
                                        ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                                        return libclang::CXChildVisit_Break;
                                    }
                                };
                                if function_name == "main" {
                                    (function_name, "".to_string())
                                } else {
                                    return libclang::CXChildVisit_Continue
                                }
                            }
                        }
                    }
                    Err(err) => {
                        ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                        return libclang::CXChildVisit_Break
                    }
                };

                let returns = 
                match rustclang::cxstring_to_string_consumable(
                    libclang::clang_getTypeSpelling(
                        libclang::clang_getResultType(
                            libclang::clang_getCursorType(
                                libclang::clang_getCursorSemanticParent(cursor)
                            )
                        )
                    )
                ) {
                    Ok(returns) => returns,
                    Err(err) => {
                        ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                        return libclang::CXChildVisit_Break;
                    }
                };

                let amount_of_arguments = 
                libclang::clang_getNumArgTypes(
                    libclang::clang_getCanonicalType(
                        libclang::clang_getCursorType(
                            libclang::clang_getCursorSemanticParent(cursor)
                        )
                    )
                );

                let args: Vec<String> = if amount_of_arguments <= 0 {
                    vec![]
                } else {
                    let mut accumulated = vec![];
                    for i in 0..(amount_of_arguments as u32) {
                        match rustclang::cxstring_to_string_consumable(
                            libclang::clang_getTypeSpelling(
                                libclang::clang_getArgType(
                                    libclang::clang_getCanonicalType(
                                        libclang::clang_getCursorType(
                                            libclang::clang_getCursorSemanticParent(cursor)
                                        )
                                    )
                                , i)
                            )
                        ) {
                            Ok(arg) => accumulated.push(arg),
                            Err(err) => {
                                ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                                return libclang::CXChildVisit_Break;
                            }
                        }
                    }
                    accumulated
                };

                let function = super::Function {
                    returns,
                    args,
                    name: full_function_name,
                    name_slice: full_cesty_name,
                };

                let range = super::Range {
                    body: body_range,
                    template: template_range
                };

                
                let comment = match config_string_from_cursor(
                    libclang::clang_getCursorSemanticParent(cursor)
                ) {
                    Ok(comment_parts) => comment_parts,
                    Err(err) => {
                        ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                        return libclang::CXChildVisit_Break;
                    }
                };

                let config = match comment {
                    Some(comment) => match Config::from_comment_lines(
                        comment, 
                        rustclang::filename_from_cursor(cursor).unwrap_or("unknown".to_owned())
                    ) {
                        Ok(config) => config,
                        Err(err) => {
                            ERROR.with(|n| (*n.borrow_mut()) = Some(debugpush!(err)));
                            return libclang::CXChildVisit_Break;
                        }
                    },
                    None => Config::default()
                };

                let test = super::ParsedTest {
                    config,
                    function,
                    range
                };

                if test.function.name == "main" {

                    let (main_line, main_column) = rustclang::position_from_cursor_location(
                        libclang::clang_getCursorSemanticParent(cursor)
                    ).0;

                    if MAIN.with(|n| n.borrow_mut().is_some()) {

                        let filename = rustclang::filename_from_cursor(cursor).unwrap_or("unknown".to_owned());

                        let code = match std::fs::read_to_string(&filename) {
                            Ok(contents) => {
                                contents.lines().collect::<Vec<&str>>()[main_line-1].to_owned()
                            }
                            Err(_) => {
                                ERROR.with(|n| (*n.borrow_mut()) = Some(Alert::Error( AlertInfo{
                                    description: format!("file `{filename}` contains multiple main() functions"),
                                    debug: debuginfo!(),
                                    example: None,
                                    note: vec![]
                                })));
                                return libclang::CXChildVisit_Break;
                            }
                        };



                        let old_main_line = MAIN.with(|n| n.borrow_mut().clone().unwrap().1);
                        let old_main_column = MAIN.with(|n| n.borrow_mut().clone().unwrap().2);

                        ERROR.with(|n| (*n.borrow_mut()) = Some(Alert::Error( AlertInfo{
                            description: format!("file contains multiple main() functions"),
                            debug: debuginfo!(),
                            example: Some(AlertExample::Code(AlertCode { 
                                line: main_line, 
                                code: code, 
                                file: filename_from_cursor(cursor).unwrap_or("unknown".to_owned()), 
                                fix: vec![
                                    AlertCodeFix {
                                        relative_line: 0,
                                        column: main_column,
                                        comment: format!("already encountered a main() on line {}, column {}", old_main_line, old_main_column)
                                    }
                                ]
                            })),
                            note: vec![]
                        })));
                        return libclang::CXChildVisit_Break;
                    } 
                    MAIN.with(|n| (*n.borrow_mut()) = Some((test, main_line, main_column)))
                } else {
                    TEST_STACK.with(|n| n.borrow_mut().push(test));
                }

                return libclang::CXChildVisit_Continue;

            }
            else
            { 
                return libclang::CXChildVisit_Continue;
            }

        }}

        unsafe { 
            libclang::clang_visitChildren(
                clang.cursor.clone(),
                filter,
                null_mut()
            );
        }

        if ERROR.with(|n| n.borrow_mut().is_some()) {

            return Err(debugpush!(ERROR.with(|n| (*n.borrow_mut()).to_owned().unwrap())));

        }

        let clean:    Vec<(usize, usize, bool)> = CLEAN_STACK.with(|n| n.borrow_mut().to_owned());
        let warnings: Vec<Alert> = debugappend!(WARNINGS.with(|n| n.borrow_mut().to_owned()));
        let tests:    Vec<ParsedTest> = TEST_STACK.with(|n| n.borrow_mut().to_owned());
        let main:     Option<(ParsedTest, usize, usize)> = MAIN.with(|n| (*n.borrow_mut()).to_owned());

        if tests.is_empty() {
            return Ok(((vec![], None, Environment::default()), warnings))
        }

        let file_contents = match std::fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(err) => return error!{
                description: "failed to extract file contents".to_owned(),
                debug: debuginfo!(),
                example: None,
                note: function_message!("std::fs::read_to_string()", err.to_string())
            }
        };

        let environment = Environment {
            full: file_contents.clone(),
            mainless: {
                let mut cleaned_up = file_contents.clone();
                if main.is_some() {
                    cleaned_up.replace_range(main.as_ref().unwrap().0.range.template.0..main.as_ref().unwrap().0.range.body.1, "");
                }
                cleaned_up
            },
            templated: {
                let mut cleaned_up = file_contents.clone();
                for cleanup in clean.iter().rev() {
                    cleaned_up.replace_range(cleanup.0..cleanup.1, if cleanup.2 == true {""} else {";"} );
                }
                cleaned_up
            }
        };

        if main.is_some() {

            Ok(((tests, Some(main.unwrap().0), environment), warnings))

        } else {

            Ok(((tests, None, environment), warnings))

        }

    }

}
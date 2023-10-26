use crate::{
    error::ErrorGroup, 
    lister::ListerFile
};

use std::{
    error::Error, 
    fmt::Display, 
    ffi::{c_void, CString, CStr}, 
    ptr::{null_mut, addr_of}, 
    path::Path
};

use clang_sys::*;
use indoc::indoc;
use colored::Colorize;

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum ClangError {
    IndexInitFailed,
    CStringConversionError(String),
    TranslationUnitInitFailed(String)
}

impl ClangError {

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

impl Display for ClangError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::IndexInitFailed => {
                format!( indoc!{"
                Error! 
                    clang_createIndex() returned NULL, aka. clang failed
                    to initialize!.
                "})
                .red()},
            Self::TranslationUnitInitFailed(str)=> {
                format!( indoc!{"
                Error! 
                    clang_parseTranslationUnits() returned NULL, aka. 
                    clang failed to translate the file, probably
                    because:
                        File: {}
                    Does not contain valid C.
                "}, 
                    str.bold().red())
                .red()}
            Self::CStringConversionError(str)=> {
                format!( indoc!{"
                Error! 
                    Was unable to initilaize a CString from this string:
                        String: {}
                "}, 
                    str.bold().red())
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

impl Error for ClangError {}

pub struct Clang {

    pub index: *mut c_void,
    pub tu: *mut c_void,
    pub cur: CXCursor

}

impl Default for Clang {

    fn default() -> Self {
        Clang {
            index: null_mut(),
            tu: null_mut(),
            cur: CXCursor::default() 
        }
    }

}

impl Clang {

    pub fn from_lister(
        file: &ListerFile
    ) -> Result<Clang, ClangError> 
    { unsafe {

        let clang = clang_createIndex(
            0, 0
        );
        
        if clang.is_null() {
            return Err(ClangError::IndexInitFailed);
        }
        
        let raw_str: CString = match CString::new(
            file.path
                .clone()
                .to_string_lossy()
                .to_string()
        ) {
            Ok(val) => {val}
            Err(_) => {
                return Err(ClangError::CStringConversionError(
                    file.path
                        .clone()
                        .to_string_lossy()
                        .to_string()
                ));
            }
        };

        let arg = CStr::from_bytes_with_nul("-std=gnu11\0".as_bytes()).unwrap();
        let argptr = arg.as_ptr();

        // let translation_unit: *mut *mut c_void = null_mut();
        let translation_unit = 
        clang_parseTranslationUnit(
            clang, 
            raw_str.as_ptr(),
            addr_of!(argptr),
            0,
            null_mut(),
            0,
            CXTranslationUnit_None
            // | CXTranslationUnit_SkipFunctionBodies
        );

        if translation_unit.is_null() {
            return Err(ClangError::TranslationUnitInitFailed(
                file.path
                    .clone()
                    .to_string_lossy()
                    .to_string())
            );
        }

        let cur = clang_getTranslationUnitCursor(translation_unit);

        _ = arg;
        _ = raw_str;

        return Ok( 
            Clang {
                index: clang,
                tu: translation_unit,
                cur: cur
            }
        );

    }}

    pub fn close(&self) {

        if !self.tu.is_null() {
            unsafe{ clang_disposeTranslationUnit(self.tu); }
        }
        if !self.index.is_null() {
            unsafe{ clang_disposeIndex(self.index); }
        }

    }

}
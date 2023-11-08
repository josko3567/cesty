use crate::{
    error::{ErrorGroup, ErrorPosition}, 
    lister::ListerFile
};

use std::{ 
    fmt::Display, 
    ffi::{c_void, CString, CStr}, 
    ptr::{null_mut, addr_of}, 
    path::Path
};

use clang_sys::*;
use indoc::formatdoc;
use colored::Colorize;

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Error {
    IndexInitFailed(ErrorPosition),
    CStringConversionError(ErrorPosition, String),
    TranslationUnitInitFailed(ErrorPosition, String)
}

impl Error {

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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::IndexInitFailed(pos) => {
                fmtperr!(pos,
                "Failed to initialize clang engine!",
                "
                   {} failed with NULL. 
                ",
                    fmterr_func!(clang_createIndex())
                )}
            Self::TranslationUnitInitFailed(pos, str)=> {
                fmtperr!(pos,
                "Failed to parse file!",
                "
                    {} failed with NULL, are you sure the file...
                        {}
                    ...contains valid C?
                ",
                    fmterr_func!(clang_parseTranslationUnit()), fmterr_val!(str)
                )}
            Self::CStringConversionError(pos, str)=> {
                fmtperr!(pos,
                "String conversion failed!",
                "
                    Could not initialize a CString from the string...
                        {}
                ",
                    fmterr_val!(str)
                )}
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

/// libclang's C engine in a structure.
/// Initialized from `Clang::from_lister()`
/// # Warning
/// Must be manually closed with `Clang::close(&self)`
pub struct Clang {

    pub index: *mut c_void,
    pub tu: *mut c_void,
    pub cur: CXCursor

}

impl Default for Clang {

    fn default() -> Self {
        Clang {
            index: null_mut(),
            tu:    null_mut(),
            cur:   CXCursor::default() 
        }
    }

}

impl Clang {

    /// Initialize libclang Abstract Syntax Tree from a ListerFile.
    pub fn from_lister(
        file: &ListerFile
    ) -> Result<Clang, Error> 
    { unsafe {

        let clang = clang_createIndex(
            0, 0
        );
        
        if clang.is_null() {
            reterr!(Error::IndexInitFailed);
        }
        
        let raw_str: CString = match CString::new(
            file.path
                .clone()
                .to_string_lossy()
                .to_string()
        ) {
            Ok(val) => {val}
            Err(_) => {
                reterr!(Error::CStringConversionError,
                    file.path
                        .clone()
                        .to_string_lossy()
                        .to_string()
                );
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
        );

        if translation_unit.is_null() {
            reterr!(Error::TranslationUnitInitFailed,
                file.path
                    .clone()
                    .to_string_lossy()
                    .to_string()
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

    /// Close up and deallocate the AST.
    pub fn close(&self) {

        if !self.tu.is_null() {
            unsafe{ clang_disposeTranslationUnit(self.tu); }
        }
        if !self.index.is_null() {
            unsafe{ clang_disposeIndex(self.index); }
        }

    }

}
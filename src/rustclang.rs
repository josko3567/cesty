//! Implementation of more advanced FFI functions for libclang.

use std::{
    ffi::{c_char, c_uint, CStr, CString},
    os::raw::c_void, 
    path::PathBuf, 
    ptr::null_mut
};

// use alloc::ffi::CString;
use clang_sys as libclang;

use crate::error::{
    debuginfo, error, function_message, Alert, AlertInfo
};

/// Wrapper for the index returned by [libclang::clang_createIndex]
/// and its translation unit returned by [libclang::clang_parseTranslationUnit].
/// 
/// Initialization
/// --------------
/// [Clang::open] is used for initialization, its implemented
/// from the trait [Open] such that more variations can be added
/// if needed.
/// 
/// Cleanup
/// -------
/// [Clang] implements the [Drop] property so cleanup is seamless.
pub struct Clang<'a> {

    pub file:             &'a PathBuf,
    pub index:            *mut c_void,
    pub translation_unit: *mut c_void,
    pub cursor:           libclang::CXCursor

}

pub trait Open<T, O> {
    fn open(path: T, args: O) -> Result<Self, Alert> where Self: Sized;
}

impl<'a> Open<&'a PathBuf, &'a str> for Clang<'a> {

    fn open(file: &'a PathBuf, args: &'a str) -> Result<Self, Alert> {

        let index = unsafe {
            let index = libclang::clang_createIndex(0, 0);
            if index.is_null() {
                return error!{
                    debug: debuginfo!(),
                    description: format!("failed to create index from file `{}`", file.to_string_lossy()),
                    example: None,
                    note: vec![]
                }
            }
            index
        };

        let c_path = CString::new(
            file.to_string_lossy().to_string()
        ).expect("Failed to convert &PathBuf into CString");

        let vec_args: Vec<String> = args
            .to_owned()
            .split_whitespace()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect();

        let mut vec_cstring_args: Vec<CString> = vec![];

        for arg in vec_args.into_iter() {

            let cstring_arg = match CString::new(arg) {
                Ok(cstring) => cstring,
                Err(err) => return error!{
                    description: "failed to convert `&str` into `CString`".to_owned(),
                    debug: debuginfo!(),
                    example: None,
                    note: function_message!("CString::new()", err.to_string())
                }
            };

            vec_cstring_args.push(cstring_arg);

        }

        let vec_pointer_args: Vec<*const c_char> = vec_cstring_args
            .iter()
            .map(|x: &_| x.as_ptr())
            .collect::<Vec<*const i8>>();

        let final_args: *const *const i8 = vec_pointer_args.as_ptr();

        let translation_unit = unsafe { 
            let tu = libclang::clang_parseTranslationUnit(
                index, 
                c_path.as_ptr(),
                final_args,
                vec_pointer_args.len() as i32,
                null_mut(),
                0,
                libclang::CXTranslationUnit_None 
                | libclang::CXTranslationUnit_DetailedPreprocessingRecord
                | libclang::CXTranslationUnit_SingleFileParse
            );
            if tu.is_null() {
                if !index.is_null() {
                    libclang::clang_disposeIndex(index);
                }
                return error!{
                    debug: debuginfo!(),
                    description: format!("failed to create translation unit from file `{}`", file.to_string_lossy()),
                    example: None,
                    note: vec![]
                }
            }
            tu
        };

        Ok(Clang {
            file,
            index,
            translation_unit,
            cursor: unsafe{libclang::clang_getTranslationUnitCursor(translation_unit)}
        })

    }

} 

impl Drop for Clang<'_> {

    fn drop(&mut self) {

        if !self.translation_unit.is_null() {
            unsafe{ libclang::clang_disposeTranslationUnit(self.translation_unit); }
        }
        if !self.index.is_null() {
            unsafe{ libclang::clang_disposeIndex(self.index); }
        }

    }
    
}

/// A translation of a [libclang::CXSourceRange] gotten
/// from [libclang::clang_getCursorExtent] into a Rust
/// range.
/// 
/// Member 0 is the start, 1 is the end.
pub fn range_from_cursor_extent(
    cursor: libclang::CXCursor
) -> (usize, usize)
{ 
    
    let mut start: u32 = 0;
    let mut end:   u32 = 0;
    
    unsafe {
        let range_cx = libclang::clang_getCursorExtent(cursor);

        libclang::clang_getFileLocation(
            libclang::clang_getRangeStart(range_cx), 
            null_mut(),
            null_mut(),
            null_mut(),
            std::ptr::addr_of_mut!(start) as *mut c_uint
        );

        libclang::clang_getFileLocation(
            libclang::clang_getRangeEnd(range_cx), 
            null_mut(),
            null_mut(),
            null_mut(), 
            std::ptr::addr_of_mut!(end) as *mut c_uint
        );
    }

    (start as usize, end as usize)

}

/// A translation of a [libclang::CXSourceRange] gotten
/// from [libclang::clang_getCursorExtent] into a 2 file 
/// positions.
/// 
/// The tuple members are as such:
/// ((start_line, start_column), (end_line, end_column))
pub fn position_from_cursor_extent(
    cursor: libclang::CXCursor
) -> ((usize, usize), (usize, usize))
{ 
    
    let mut start_collumn: u32 = 0;
    let mut start_line:    u32 = 0;

    let mut end_collumn: u32 = 0;
    let mut end_line:    u32 = 0;
    
    unsafe {

        let range_cx = libclang::clang_getCursorExtent(cursor);

        libclang::clang_getFileLocation(
            libclang::clang_getRangeStart(range_cx), 
            null_mut(), 
            std::ptr::addr_of_mut!(start_line) as *mut c_uint,
            std::ptr::addr_of_mut!(start_collumn) as *mut c_uint,
            null_mut()
        );

        libclang::clang_getFileLocation(
            libclang::clang_getRangeEnd(range_cx), 
            null_mut(), 
            std::ptr::addr_of_mut!(end_line) as *mut c_uint,
            std::ptr::addr_of_mut!(end_collumn) as *mut c_uint,
            null_mut()
        );

    }

    ((start_line as usize, start_collumn as usize), (end_line as usize, end_collumn as usize))

}

/// A translation of a [libclang::CXSourceRange] gotten
/// from [libclang::clang_getCursorLocation] into a 2 file 
/// positions.
/// 
/// Usually smaller as a [libclang::clang_getCursorExtent]
/// also includes the children of the cursor as the range.
/// 
/// The tuple members are as such:
/// ((start_line, start_column), (end_line, end_column))
pub fn position_from_cursor_location(
    cursor: libclang::CXCursor
) -> ((usize, usize), (usize, usize))
{ 
    
    let mut start_collumn: u32 = 0;
    let mut start_line:    u32 = 0;

    let mut end_collumn: u32 = 0;
    let mut end_line:    u32 = 0;
    
    unsafe {

        let range_cx = libclang::clang_getCursorLocation(cursor);

        libclang::clang_getFileLocation(
            range_cx, 
            null_mut(), 
            std::ptr::addr_of_mut!(start_line) as *mut c_uint,
            std::ptr::addr_of_mut!(start_collumn) as *mut c_uint,
            null_mut()
        );

        libclang::clang_getFileLocation(
            range_cx, 
            null_mut(), 
            std::ptr::addr_of_mut!(end_line) as *mut c_uint,
            std::ptr::addr_of_mut!(end_collumn) as *mut c_uint,
            null_mut()
        );

    }

    ((start_line as usize, start_collumn as usize), (end_line as usize, end_collumn as usize))

}


/// Given a [libclang::CXCursor] find its origin file.
pub fn filename_from_cursor(
    cursor: libclang::CXCursor
) -> Result<String, Alert>
{ 

    let mut file: libclang::CXFile = null_mut();
            
    unsafe {

        let location = libclang::clang_getCursorLocation(cursor);

        libclang::clang_getFileLocation(
            location,
            std::ptr::addr_of_mut!(file) as *mut libclang::CXFile, 
            null_mut(), 
            null_mut(), 
            null_mut()
        );

        return cxstring_to_string_consumable(libclang::clang_getFileName(file))
        
    }

}

/// Converts a [libclang::CXString] into rusts [String] whilst
/// consuming the original [libclang::CXString] (aka no need for 
/// [libclang::clang_disposeString]).
pub fn cxstring_to_string_consumable(
    s: libclang::CXString
) -> Result<String, Alert> {

    if s.data.is_null() {
        unsafe{libclang::clang_disposeString(s)};
        return error!{
            description: "libclang: `CXString::data` was null while converting to `String`".to_owned(),
            debug: debuginfo!(),
            example: None,
            note: vec![
                "most likely a developer issue, publish this issue along with the file where the issue occurs.".to_owned()
            ]
        }
    }

    let converted = unsafe{CStr::from_ptr(s.data as *const i8)}.to_string_lossy().to_string();
    unsafe{libclang::clang_disposeString(s)};
    Ok(converted)

}
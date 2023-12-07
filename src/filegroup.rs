use strum_macros::*;
use std::str::FromStr;
use std::process::exit;
use std::path::Path;
use colored::Colorize;
use indoc::formatdoc;

/// FileGroup is enum that contains all source files.
/// Every file has a corresponding enum value, 
/// [strum EnumString](EnumString) and [strum EnumProperty's](EnumProperty).
/// This way it is easier to give certain files special properties like
/// with the [clearance](crate::globals::AccessLevel) property.
#[repr(u8)]
#[allow(dead_code)]
#[derive(EnumString, EnumProperty, Default, Debug)]
pub enum FileGroup {
    #[strum(serialize="null")]
    // Clearance of negative is a mistake, 0 means Unset, 1 Config,
    // 2 Argument, 3 Main, 4 Overwrite
    #[strum(props(clearance="0"))]
    #[default]
    Unknown,

    #[strum(serialize="main.rs")]
    #[strum(props(clearance="3"))]
    Main,

    #[strum(serialize="filegroup.rs")]
    #[strum(props(clearance="0"))]
    FileGroup,

    #[strum(serialize="argument.rs")]
    #[strum(props(clearance="2"))]
    Argument,

    #[strum(serialize="config.rs")]
    #[strum(props(clearance="1"))]
    Config,

    #[strum(serialize="lister.rs")]
    #[strum(props(clearance="0"))]
    Lister,

    #[strum(serialize="extract.rs")]
    #[strum(props(clearance="0"))]
    Extract,

    #[strum(serialize="clang.rs")]
    #[strum(props(clearance="0"))]
    Clang,

    #[strum(serialize="environment.rs")]
    #[strum(props(clearance="0"))]
    Environment,

    #[strum(serialize="translate.rs")]
    #[strum(props(clearance="0"))]
    Translate

}

impl FileGroup {
    
    /// Get the corresponding FileGroup for the current source file.
    /// # Abnormal exit
    /// ... occurs when the filename given to this function
    /// does not exist.
    pub fn from(filename: &str) -> Self {
        
        match FileGroup::from_str(filename) {
            Ok(val) => {val}
            _ => {
                #[cfg(debug_assertions)] {
                    let err = fmterr!("Debug: Unknown value.",
                    "
                        Unknown value {} for {}.
                    ", filename, fmterr_func!(FileGroup::from_str(value))
                    );
                    eprintln!("{}", err);
                    exit(1);
                }
                unreachable!();
            }
        }
        
    }

}

/// Get current files filename. 
macro_rules! filename {
    () => {
        Path::new(file!())
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap()
    };
}
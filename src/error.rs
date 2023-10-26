use std::str::FromStr;
use strum_macros::*;

#[repr(u8)]
#[allow(dead_code)]
#[derive(EnumString)]
pub enum ErrorGroup {
    #[strum(serialize="unknown file")]
    Unknown,

    #[strum(serialize="argument.rs")]
    Argument,

    #[strum(serialize="config.rs")]
    Config,

    #[strum(serialize="lister.rs")]
    Lister,

    #[strum(serialize="extract.rs")]
    Extract,

    #[strum(serialize="clang.rs")]
    Clang,

    #[strum(serialize="environment.rs")]
    Environment,

    #[strum(serialize="translate.rs")]
    Translate

}

impl ErrorGroup {
    
    pub fn from(value: &str) -> Self {
    
        match ErrorGroup::from_str(value) {
            Ok(val) => {val}
            _ => {ErrorGroup::Unknown}
        }
        
    }

}
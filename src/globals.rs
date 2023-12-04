use std::sync::RwLock;

use lazy_static::lazy_static;

use strum::EnumProperty;
use strum_macros::Display;

use crate::filegroup::FileGroup;

// Specific ordering, Higher number means higher priority.
#[repr(u8)]
#[allow(dead_code)]
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub enum AccessLevel {
    #[default]
    Unset,
    Config,
    Argument,
    Main,
    Overwrite
}

impl AccessLevel {

    pub fn value(&self) -> u8 {
        unsafe { *(self as *const Self as *const u8) }
    }

    pub fn from_filename(filename: &str) -> Self {

        let filegr = FileGroup::from(filename);

        match filegr {
            FileGroup::Unknown => {AccessLevel::Unset}
            _ => {

                let Some(clearance_str) = filegr.get_str("clearance")
                else {
                    return AccessLevel::Unset;
                };

                let Ok(clearance) = clearance_str.parse::<i32>()
                else {
                    return AccessLevel::Unset;
                };

                match clearance {
                    4 => AccessLevel::Overwrite,
                    3 => AccessLevel::Main,
                    2 => AccessLevel::Argument,
                    1 => AccessLevel::Config,
                    _ => AccessLevel::Unset,
                }
                
            }
        }

    }

    // fn from_i32()

}


#[derive(Debug, Display, Clone, Default)]
pub enum Degree {
    #[default]
    Small,
    Medium,
    Large
}

impl From<&char> for Degree {

    fn from(value: &char) -> Self {
        match value {
            's' | 'S' => {Degree::Small}
            'm' | 'M' => {Degree::Medium}
            'l' | 'L' => {Degree::Large}
            _ => {Degree::default()}
        }
    }
}

impl From<&String> for Degree {

    fn from(value: &String) -> Self {
        match value.to_lowercase().as_str() {
            "small"  | "s" => {Degree::Small}
            "medium" | "m" => {Degree::Medium}
            "large"  | "l" => {Degree::Large}
            _ => {Degree::default()}
        }
    }
}

impl TryFrom<char> for Degree {

    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            's' | 'S' => {Ok(Degree::Small )}
            'm' | 'M' => {Ok(Degree::Medium)}
            'l' | 'L' => {Ok(Degree::Large )}
            _ => {Err(())}
        }
    }

}

impl TryFrom<&str> for Degree {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        
        match value.to_lowercase().as_str() {
            "small"  | "s" => {Ok(Degree::Small)}
            "medium" | "m" => {Ok(Degree::Medium)}
            "large"  | "l" => {Ok(Degree::Large)}
            _ => {Err(())}
        }

    }

}

#[derive(Clone)]
pub struct Globals {
    //              Value,   Was set & by whom
    warn:           (bool,   AccessLevel),
    message_amount: (Degree, AccessLevel),

}

impl Default for Globals {

    fn default() -> Self {
        Globals { 
            warn:           (false,             AccessLevel::default()), 
            message_amount: (Degree::default(), AccessLevel::default())
        }
    }

}

#[allow(dead_code)]
impl Globals {

    // Globals.warn
    pub fn get_warn(&self) -> bool {self.warn.0}
    pub fn set_warn(&mut self, v: bool, from: AccessLevel){

        if from.value() >= self.warn.1.value() {
            self.warn.0 = v;
            if from != AccessLevel::Overwrite {self.warn.1 = from}
        }

    }

    // Globals.message_amount
    pub fn get_message_amount(&self) -> Degree {self.message_amount.0.clone()}
    pub fn set_message_amount(&mut self, v: Degree, from: AccessLevel){
 
        if from.value() >= self.message_amount.1.value() {
            self.message_amount.0 = v;
            if from != AccessLevel::Overwrite {self.message_amount.1 = from}
        }

    }

}

lazy_static!{
    pub static ref GLOBALS: RwLock<Globals> = RwLock::new(Globals::default());
}
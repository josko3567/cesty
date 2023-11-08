use std::sync::RwLock;
use lazy_static::lazy_static;
use strum_macros::Display;

#[derive(Debug, Display)]
pub enum Degree {
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
            _ => {Degree::Small}
        }
    }

}

pub struct Globals {

    pub warn:    bool,
    pub message: Degree,

}

impl Default for Globals {

    fn default() -> Self {
        Globals { 
            warn: false, 
            message: Degree::Small 
        }
    }

}

lazy_static!{
    pub static ref GLOBALS: RwLock<Globals> = RwLock::new(Globals::default());
}
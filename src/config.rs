use crate::{
    argument::{Argument, Override}, 
    error::{ErrorGroup, ErrorPosition}
};

use serde::Deserialize;
// use serde::
use serde_yaml;

use std::{
    fs::File, 
    path::Path,
    fmt::Display
};

use indoc::formatdoc;
use colored::Colorize;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Error {
    NoConfigFile(ErrorPosition),
    CannotOpenConfig(ErrorPosition, String),
    SerdeError(ErrorPosition, String)
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
            Self::NoConfigFile(pos) => {
                fmtwarnp!(pos,
                "No config file was found.",
                "
                    Proceeding with argument passed files.
                ")}
            Self::CannotOpenConfig(pos, str) => {
                fmtperr!(pos,
                "Cannot open file.",
                "
                    Config file exists, but std::io functionality cannot open
                    it for the following reason...
                        {}
                ",
                    str.bold()
                )}
            Self::SerdeError(pos, str) => {
                fmtperr!(pos,
                "Cannot parse the config file.",
                "
                    Serde failed to parse cesty config file for the following
                    reason...
                        {}
                ",
                    str.bold()
                )}
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigCestyData {
    #[serde(rename = "use")] 
    pub active: Option<bool>,
    #[serde(alias = "input")]
    pub output: Option<String>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigCesty {
    
    pub flags:     Option<String>,
    pub metadata:  Option<ConfigCestyData>,
    pub dataset:   Option<ConfigCestyData>

}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigCompiler {

    pub name:      Option<String>,
    pub flags:     Option<String>,
    pub libraries: Option<String>

}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigRecipeRun {

    pub path:     String,
    pub recurse:  Option<bool>,
    pub symlinks: Option<bool>

}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigRecipe {

    pub name:      String,
    pub run:       Vec<ConfigRecipeRun>,
    pub force:     Option<bool>

}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {

    #[serde(skip_deserializing)]
    pub path:     String,

    pub cesty:    Option<ConfigCesty>,
    pub compiler: Option<ConfigCompiler>,
    pub recipe:   Option<Vec<ConfigRecipe>>

}

/// Find the first occurence of .cesty.{conf, yaml, yml} from the current file
/// all the way to root (C:\ or /).
/// 
/// # Example
/// 
/// ```
/// // If /home/foobar/.cesty.yaml exists but we are in /home/foobar/bar/
/// // find_config will find /home/foobar/.cesty.yaml.
/// assert_eq!(find_config(), "/home/foobar/.cesty.yaml");
/// 
/// // If we add a .cesty.yaml to our current folder we will find the one
/// // in that folder.
/// assert_eq!(find_config(), "/home/foobar/bar/.cesty.yaml");
/// ```
pub fn find() -> Option<String> {
 
    let names = [

        "cesty.conf",
        "cesty.yaml",
        "cesty.yml"

    ].into_iter();

    for name in names {

        let mut current_full_path = 
        match std::env::current_dir() {
            Ok(res) => {res}
            Err(_) => {
                return None
            }
        };


        loop {

            let mut cfp_clone = current_full_path.clone();
            cfp_clone.push(Path::new(name));

            let test_path = cfp_clone
                .to_string_lossy()
                .to_string();

            if Path::new(&test_path).is_file() {
                
                return Some(test_path);
                
            }

            if !current_full_path.pop() {break}

        }

    }

    return None;
        
}

impl Config {

    #[allow(non_snake_case)]
    pub fn new() -> Config {

        let configCestyMetaDataEmpty: ConfigCestyData = ConfigCestyData { 
            active: None, 
            output: None
        };

        let configCestyUserDataEmpty: ConfigCestyData = ConfigCestyData { 
            active: None, 
            output: None
        };

        let configCestyEmpty: ConfigCesty = ConfigCesty { 
            flags: None, 
            metadata: Some(configCestyMetaDataEmpty), 
            dataset: Some(configCestyUserDataEmpty) 
        };

        let configCompilerEmpty: ConfigCompiler = ConfigCompiler { 
            name: Some("gcc".to_string()), 
            flags: None, 
            libraries: None
        };

        Config {

            path: String::from(""),
            cesty: Some(configCestyEmpty),
            compiler: Some(configCompilerEmpty),
            recipe: None

        }

    }


    /// Read config file from optional string.
    /// Used in tandem with [`config::find()`](find())
    /// # Example
    /// ```
    /// let mut conf: Config = Config::new();
    /// match conf.from_file(config::find()) {
    ///     Err(err) => {eprintln!("{err}"); return Err(err.code())}
    ///     _ => {}
    /// }
    /// ```
    pub fn from_file(&mut self, file: Option<String>) -> Result<(), Error> {

        let filepure = match file {
            Some(pure) => {pure}
            None => {reterr!(Error::NoConfigFile)}
        };

        let stream = match File::options()
            .truncate(false)
            .append(false)
            .write(false)
            .read(true)
            .open(&filepure)
        {
            Ok(res) => {res}
            Err(err) => {reterr!(Error::CannotOpenConfig, err.to_string())}
        };

        let parsed: Result<Config, serde_yaml::Error> = 
            serde_yaml::from_reader(stream);
        
        *self = match parsed {
            Ok(res) => {res}
            Err(err) => {reterr!(Error::SerdeError, err.to_string())}
        };

        self.path = String::from(filepure);

        Ok(())

    }

    pub fn merge_overrides(&mut self, args: &Vec<Argument>) {

        let iter = args.into_iter();
        for arg in iter {

            match arg {
                Argument::Overrides(
                    Override::CompilerName(name)
                ) => {
                    match self.compiler.as_mut() {
                        Some(compiler) => {
                            compiler.name = Some(name.to_string());
                        }
                        None => {}
                    }
                },
                Argument::Overrides(
                    Override::CompilerFlags(flags)
                ) => {
                    match self.compiler.as_mut() {
                        Some(compiler) => {
                            compiler.flags = Some(flags.to_string());
                        }
                        None => {}
                    }
                },
                Argument::Overrides(
                    Override::CompilerLibraries(libs)
                ) => {
                    match self.compiler.as_mut() {
                        Some(compiler) => {
                            compiler.libraries = Some(libs.to_string());
                        }
                        None => {}
                    }
                },
                _ => {
                    continue;
                }

            }

        }

    } 

}
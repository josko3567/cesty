use crate::{
    argument::{Argument, Override}, 
    error::ErrorPosition,
    filegroup::FileGroup,
    globals::{GLOBALS, self}
};

use serde::Deserialize;
// use serde::
use serde_yaml;

use std::{
    fs::File, 
    path::Path,
    fmt::Display, ffi::OsString
};

use indoc::formatdoc;
use colored::Colorize;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Error {
    NoConfigFile(ErrorPosition),
    CannotOpenConfig(ErrorPosition, String),
    SerdeError(ErrorPosition, String),
    CannotObtainPWD(ErrorPosition, String)
}

impl Error {

    pub fn code(&self) -> String {
        return format!("E;{:X}:{:X}", 
            FileGroup::from(filename!()) as u8, 
            unsafe { *(self as *const Self as *const u8) }
        );
    }
    
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::NoConfigFile(pos) => {
                if GLOBALS.read().unwrap().get_warn() { fmtpwarn!(pos,
                "No config file was found.",
                "
                    Proceeding with argument passed files.
                ")}
                else {
                    "".white()
                }}
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
            Self::CannotObtainPWD(pos, str) => {
                fmtperr!(pos,
                "Cannot obtain PWD.",
                "
                    Failed to obtain current PWD due to the following reason:
                        Error: {}
                ",
                    str.bold().bright_blue()
                )}
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ConfigCestyData {
    #[serde(rename = "use")] 
    pub active: Option<bool>,
    #[serde(alias = "input")]
    pub output: Option<String>
}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ConfigCesty {
    
    pub message: Option<String>,
    pub warn: Option<bool>,
    pub metadata:  Option<ConfigCestyData>,
    pub dataset:   Option<ConfigCestyData>

}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ConfigCompiler {

    pub name:      Option<String>,
    pub flags:     Option<String>,
    pub libraries: Option<String>

}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ConfigRecipeRun {

    pub path:     String,
    pub recurse:  Option<bool>,
    pub symlinks: Option<bool>

}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct ConfigRecipe {

    pub name:      String,
    pub run:       Vec<ConfigRecipeRun>,
    pub force:     Option<bool>,
    pub prerun:    Option<String>

}

#[derive(Debug, Deserialize, PartialEq, Default)]
pub struct Config {

    #[serde(skip_deserializing)]
    pub path:     Option<OsString>,

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
pub fn find() -> Option<OsString> {
 
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
                .as_os_str()
                .to_os_string();

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
            message: None,
            warn: None,
            metadata: Some(configCestyMetaDataEmpty), 
            dataset: Some(configCestyUserDataEmpty) 
        };

        let configCompilerEmpty: ConfigCompiler = ConfigCompiler { 
            name: Some("gcc".to_string()), 
            flags: None, 
            libraries: None
        };

        Config {

            path: None,
            cesty: Some(configCestyEmpty),
            compiler: Some(configCompilerEmpty),
            recipe: None

        }

    }

    fn set_globals(&self) {

        if self.cesty.is_some() {

            _ = self.cesty.as_ref().unwrap().warn.as_ref().is_some_and(
                |x| {
                    GLOBALS.write().unwrap().set_warn(
                           x.to_owned(), 
                           globals::AccessLevel::from_filename(filename!())
                    );
                    true
            });

            _ = self.cesty.as_ref().unwrap().message.as_ref().is_some_and(
                |x| {
                    match globals::Degree::try_from(x.as_str())
                    {
                        Ok(res) => {
                            GLOBALS.write().unwrap().set_message_amount(
                                res.clone(), 
                            globals::AccessLevel::from_filename(filename!())
                            );
                        }
                        Err(_) => {
                            if GLOBALS.read().unwrap().get_warn() { warn!(
                            "Unexpected value from config!",
                            "
                                Unexpected value...
                                  {}
                                ... for {}.
                            ",
                                fmterr_val!(x),
                                fmterr_func!("cesty: message: ...")
                            )}
                        }
                    };
                    true
            });

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
    pub fn from_file(&mut self, file: Option<OsString>) -> Result<(), Error> {

        let filepure = match file {
            Some(res) => {
                if GLOBALS.read().unwrap().get_noconfig() {
                    reterr!(Error::NoConfigFile)
                } else {
                    res
                }
            }
            None => {
                GLOBALS.write().unwrap().set_noconfig(
                    true, 
                    globals::AccessLevel::from_filename(filename!())
                );
                reterr!(Error::NoConfigFile)
            }
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

        self.path = Some(filepure.to_owned());
        self.set_globals();
        
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
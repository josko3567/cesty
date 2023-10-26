use crate::{
    argument::{Argument, Override}, 
    error::ErrorGroup
};

use serde::Deserialize;
// use serde::
use serde_yaml;

use std::{
    fs::File, 
    path::Path,
    error::Error,
    fmt::Display
};

use indoc::indoc;
use colored::Colorize;

const ERROR_GROUP: u32 = 2;

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigCestyData {
    #[serde(rename = "use")] 
    pub active: Option<bool>,
    pub output: Option<String>
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct ConfigCesty {
    
    pub flags:     Option<String>,
    pub metadata:  Option<ConfigCestyData>,
    pub userdata:  Option<ConfigCestyData>

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

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ConfigError {
    NoConfigFile,
    CannotOpenConfig,
    SerdeError
}

impl ConfigError {

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

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::NoConfigFile => {
                format!( indoc!{"
                Message: 
                    Config file was not found.
                    Proceeding with argument passed files only.
                "})
                .normal().dimmed()},
            Self::CannotOpenConfig => {
                format!( indoc!{"
                Error! 
                    Config file exists but IO functionality cannot open it.
                "})
                .red()},
            Self::SerdeError => {
                format!( indoc!{"
                Error! 
                    Serde had trouble parsing the config file.
                    Check if the file is properly formatted.
                    If you have recipes with no names the parser will fail.
                "})
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

impl Error for ConfigError {}

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
pub fn find_config() -> Option<String> {
 
    let names = [

        ".cesty.conf",
        ".cesty.yaml",
        ".cesty.yml"

    ].into_iter();

    for name in names {

        let Ok(mut current_full_path) = std::env::current_dir() else {
            // Never happens.
            continue;
        };

        loop {

            let mut cfp_clone = current_full_path.clone();
            cfp_clone.push(Path::new(name));

            let test_path = cfp_clone
                .to_string_lossy()
                .to_string();

            if Path::new(&test_path).is_file() {
                
                // eprintln!("{}", testpath);
                return Some(test_path);
                
            }

            if !current_full_path.pop() {break;}

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
            userdata: Some(configCestyUserDataEmpty) 
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

    pub fn from_file(&mut self, file: Option<String>) -> Result<(), ConfigError> {


        // serde_yaml::Error::

        let filepure = match file {
            Some(pure) => {pure}
            None => {return Err(ConfigError::NoConfigFile)}
        };


        let Ok(stream) = File::options()
            .truncate(false)
            .append(false)
            .write(false)
            .read(true)
            .open(&filepure)
        else {
            return Err(ConfigError::CannotOpenConfig)
        };

        let parsed: Result<Config, serde_yaml::Error> = 
            serde_yaml::from_reader(stream);
        
        *self = match parsed {
            Ok(pure) => {pure}
            Err(_err) => {
                return Err(ConfigError::SerdeError);
            }
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
                    // self.cesty;
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
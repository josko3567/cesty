//! Module for extracting, compiling and running tests
//! from files returned by lister.
//! 
//! # Use 
//! * [`extract`] - Used to extract tests, environments for tests
//!                 and 

pub mod extract;
pub mod compilable;

use serde::Deserialize;
use crate::{
    arg_conf::serde_tokenize_strings_and_vec, 
    error::{
        debuginfo, error, Alert, 
        AlertCode, AlertCodeFix, 
        AlertExample, AlertInfo
    }
};

/// Appends `flags` & `libraries` for test compilation/linking.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct CompilerAppend {

    /// What flags to use while compiling/linking.
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[serde(default = "Vec::new")]
    pub flags: Vec<String>,

    /// What libraries to use while compiling/linking.
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[serde(default = "Vec::new")]
    pub libraries: Vec<String>

}

/// Structure for replacing a flag/library with a new flag.
/// - `old` can be a string or a regex to locate the old flag/library.
/// - `new` is the replacement string.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct CompilerReplaceItem {

    /// Can be a string or a regex to match the old flag/library.
    pub old: String,

    /// The replacement string.
    pub new: String

}

/// Struct used to separate members `flag` & `library` into
/// a different namespace for the sake of readability in the
/// markdown language.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct CompilerReplace {

    /// Vector of replacements to be done for the flags.
    #[serde(default = "Vec::new")]
    pub flag:    Vec<CompilerReplaceItem>,

    /// Vector of replacements to be done for the libraries.
    #[serde(default = "Vec::new")]
    pub library: Vec<CompilerReplaceItem>

}

/// Global compiler settings read from the config file.
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Compiler {

    /// A C compiler to use while compiling/linking.
    pub name: Option<String>,

    /// What flags to use while compiling/linking.
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[serde(default = "Vec::new")]
    pub flags: Vec<String>,

    /// What libraries to use while compiling/linking.
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[serde(default = "Vec::new")]
    pub libraries: Vec<String>,
    
    /// Append flags/libraries to the existing set of flags/libraries.
    pub append: Option<CompilerAppend>,

    /// Replaces a flag/library based on matching regex. 
    pub replace: Option<CompilerReplace>

}

/// Used for default values.
fn settings_bool_init() -> bool {false}

/// Settings available to the test comment markup.
#[derive(Deserialize, Clone, Debug)]
pub struct Settings {

    // /// Simply put, if the code inside your test is contained 
    // /// within the file / any included files and does not require
    // /// any external code dependencies, set this to true. 
    // /// 
    // /// Otherwise if you need to include the compiled code
    // /// then "standalone" is false.
    // #[serde(default = "settings_bool_init")]
    // pub standalone: bool,

    /// Run the test or do not run the test, overwritten
    /// with the -f / --force flag. 
    #[serde(default = "settings_bool_init")]
    pub run: bool,

    /// Let stdout be displayed while the test is running.
    #[serde(default = "settings_bool_init")]
    pub stdout: bool,
    
    /// Allow user input through stdin while the test is running.
    #[serde(default = "settings_bool_init")]
    pub stdin: bool

}

impl Default for Settings {

    fn default() -> Self {
        Settings {
            run:    true,
            stdout: false,
            stdin:  false
        }
    }

}

/// Config written via markup (TOML) read from the
/// test function comment. 
#[derive(Deserialize, Clone, Debug, Default)]
pub struct Config {

    /// Settings to tweak the way cesty runs on the test.
    #[serde(default = "Settings::default")]
    pub settings: Settings,

    /// Options that overwrite the global compiler options.
    #[serde(default = "Compiler::default")]
    pub compiler: Compiler,

    /// Commands to run before the test.
    #[serde(default = "Vec::new")]
    pub commands: Vec<String>

}

impl Config {

    fn from_comment_lines(comment_lines: Vec<(String, String, usize, usize)>, path: String) -> Result<Self, Alert> {

        let comment = comment_lines
            .iter()
            .map(|x|x.0.to_owned()).collect::<Vec<String>>()
            .join("\n");

        let result: Config = match toml::from_str(comment.as_str()) {

            Ok(res) => res,
            Err(err) => {
                
                let mut message = err
                    .message()
                    .to_owned()
                    .replace("\n", " ")
                    .trim()
                    .to_owned();

                match err.span() {

                    Some(span) => {

                        let start_lines = (&comment[0..span.start])
                            .lines()
                            .collect::<Vec<&str>>();

                        if start_lines.is_empty() {
                            return error!{
                                debug: debuginfo!(),
                                description: "failed to parse TOML from comment into a Config type.".to_owned(),
                                example: None,
                                note: vec![
                                    "no error span was recovered, here is the error message:".to_owned(),
                                    message
                                ]
                            }
                        }

                        let line = start_lines.len()-1;

                        let column = if start_lines.last().is_some() {
                            start_lines.last().unwrap().len()
                        } else {
                            0
                        };

                        message = message + format!(" on line {}, column {}.", comment_lines[line].2, comment_lines[line].3 + column).as_str();

                        return error!{
                            debug: debuginfo!(),
                            description: "failed to parse TOML from comment into a Config type.".to_owned(),
                            example: Some(AlertExample::Code(AlertCode {
                                line: comment_lines[line].2,
                                file: path,
                                code: comment_lines[line].1.to_owned(),
                                fix:  vec![AlertCodeFix {
                                    relative_line: 0,
                                    column: column+comment_lines[line].3,
                                    comment: message
                                }]
                            })),
                            note: vec![]
        
                        }

                    }

                    None => {

                        return error!{
                            debug: debuginfo!(),
                            description: "failed to parse TOML from comment into a Config type.".to_owned(),
                            example: None,
                            note: vec![
                                "no error span was recovered, here is the error message:".to_owned(),
                                message
                            ]
        
                        }

                    }
        
                };

            }

        };

        Ok(result)



    }


}
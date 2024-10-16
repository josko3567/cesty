//! Argument and config parsing.
//! 
//! Note that when [Config::parse_cli_and_file] finishes it sets up
//! some enviornment variables (current working directory) for cesty to run
//! correctly.
//! 
//! Specificaly it sets up [std::env::current_dir] to point to the directory 
//! where the config was found.

use clap::{
    Parser, Args,
    Subcommand
};

use path_clean::PathClean;
use serde::{
    de, Deserialize, 
    Deserializer, Serialize
};

use globwalk::{
    glob_builder, DirEntry, 
    FileType, WalkError
};

use strum_macros;

use strum::{
    EnumProperty, 
    IntoEnumIterator
};

use std::{
    env, vec, fmt, 
    path::PathBuf, 
    marker::PhantomData,
    ffi::OsStr
};

use crate::defaults::{get_max_depth, CONFIG_FOLLOWUP_NAME};

use crate::error::{
    debuginfo, debugpush, error, 
    function_message, warning,
    Alert, AlertInfo
};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
#[clap(next_line_help = true)]
/// Configuration for `cesty`.
/// Can be read from the CLI and/or config file.
/// 
/// Initalizer
/// -------
/// ```
/// let config = Config::parse_from_cli_and_file();
/// ```
pub struct Config {

    #[command(subcommand)]
    pub command: Commands

}

/// Global compiler settings read from the config file.
#[derive(Serialize, Deserialize, Args, Clone, Debug, Default)]
pub struct CompilerConfig {

    #[arg(long = "compiler.name", default_value = None)]
    #[clap(allow_hyphen_values(true))]
    /// A C compiler to use while compiling/linking.
    pub name:  Option<String>,

    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[arg(long = "compiler.flags")]
    #[clap(value_delimiter = ' ')]
    #[clap(allow_hyphen_values(true))]
    /// What flags to use while compiling/linking.
    pub flags: Vec<String>,

    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    #[clap(long = "compiler.libraries")]
    #[clap(value_delimiter = ' ')]
    #[clap(allow_hyphen_values(true))]
    /// What libraries to use while compiling/linking.
    pub libraries: Vec<String>
    
}

/// A path to parse for the recipe.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ParsePath {

    pub path: PathBuf,
    
    #[serde(rename = "recurse")]
    pub recursive: Option<bool>

}

/// Compiler settings for individual recipes.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RecipeCompilerConfig {

    pub name: Option<String>,
    
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    pub flags: Vec<String>,
    
    #[serde(deserialize_with = "serde_tokenize_strings_and_vec")]
    pub libraries: Vec<String>

}

/// A recipe to read.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Recipe {

    /// The recipe name which to detect it by.
    pub name: String,
    
    /// Force run all tests.
    pub force: Option<bool>,

    /// Shell commands to prerun.
    pub prerun: Vec<String>,

    #[serde(rename = "parse")] 
    /// Paths/files to parse.
    pub parse_path: Vec<ParsePath>

}

/// Config settings & arguments for "cesty run ..." merged.
#[derive(Serialize, Deserialize, Args, Clone, Debug, Default)]
pub struct Run {

    #[serde(skip)]
    #[arg(name = "recipe")]
    /// What recipe to use from the config.
    pub recipe_name: Option<String>,

    #[arg(short = 'n', long = "noconfig")]
    #[serde(skip)]
    /// Skip parsing the config.
    pub no_config: bool,

    #[arg(long = "list")]
    #[serde(skip)]
    /// Terminate just after collecting all files to parse
    /// and instead of parsing print them to the terminal.
    pub list_paths: bool,

    #[arg(short = 'D', long = "directory")]
    #[serde(skip)]
    /// Change the current PWD.
    pub directory: Option<PathBuf>,

    #[arg(short = 'F', long = "files", num_args(0..))]
    #[serde(skip)]
    /// Add more files to parse and test.
    pub files: Vec<PathBuf>,

    #[arg(short = 'C', long = "config")]
    #[serde(skip)]
    /// Use a different config name, or specify its full/relative path.
    /// This gets written to by either `--config ...` or the config found
    /// by [`find_config`].
    pub config_path: Option<PathBuf>,

    #[command(flatten)]
    pub compiler: Option<CompilerConfig>,

    #[clap(skip)]
    pub recipes: Vec<Recipe>

}

/// Config initialization options.
#[derive(Args, Clone, Debug, Default)]
pub struct InitConfigOptions {

    /// The directory to initialize in.
    pub directory: Option<PathBuf>,

    /// The directory to initialize in.
    #[arg(short = 'N', long = "name")]
    pub name: Option<String>,

    #[arg(short = 'f', long = "force")]
    /// Forcibly initialize inside the current directory.
    pub force: bool,

    #[arg(short = 'c', long = "clean")]
    /// Removes the ".cesty" private directory if it exists.
    pub clean: bool

}

/// Type of config to be initialized, also used to recognize
/// the type of found config.
#[derive(
    Subcommand, Clone, Debug, 
    strum_macros::EnumProperty, 
    strum_macros::EnumIter, 
    strum_macros::Display, 
    strum_macros::EnumString
)]
pub enum ConfigLanguage {

    #[strum(props(extensions = "toml"))]
    /// Create a brand new TOML config.
    TOML(InitConfigOptions),

    #[strum(props(extensions = "yaml yml"))]
    /// Create a brand new YAML config.
    YAML(InitConfigOptions)

}

/// All the available "cesty [commands]" like "cesty run"
#[derive(Subcommand, Clone, Debug)]
pub enum Commands {

    /// Run cesty inside the current PWD (or some other PWD specified with args).
    Run(Run),

    /// Initalize a cesty config file.
    #[clap(subcommand)]
    Init(ConfigLanguage),

}

impl Run {
    
    fn extract_recipe(&self) -> Option<&Recipe> {
        
        if self.recipe_name == None {return None}
        let recipe_name = self.recipe_name.clone().unwrap();
        
        for recipe in self.recipes.iter() {
            
            if recipe.name == recipe_name {
                return Some(recipe)
            }
            
        }
        None
        
    }
    
}

/// Attempt to find a config in the current pwd or any of 
/// the lower level directories.
/// 
/// Arguments
/// ---------
/// - `recursive` go recursively downwards in the directory 
///               structure to find the config.
/// 
/// File Matching
/// -------------
/// The config matches with any file with the following glob pattern:
/// 
/// __*.[CONFIG_FOLLOWUP_NAME].{[ConfigLanguage::get_all_extensions].join(",")}__
/// 
/// Here are some valid samples that match:
/// - _file.cesty.yml_
/// - _config.cesty.toml_
/// - _settings.cesty.yaml_
/// 
/// File extensions that can match are returned by
/// [`ConfigLanguage::get_all_extensions()`].
/// 
/// Multiple Matched Files
/// ----------------------
/// If more than one file was found in the directory an [`Err()`]
/// is thrown.
/// 
/// No Matched Files
/// ----------------
/// If no file is matched, a [`Ok()`] containing a [`None`] is returned.
pub(super) fn find_config(config_path: Option<PathBuf>, recursive: bool) -> Result<(Option<PathBuf>, Vec<Alert>), Alert> {
    
    let mut warnings: Vec<Alert> = vec![];
    
    if config_path.is_some() {
        
        let config = config_path.unwrap();
        if !config.exists() {
            return error!{
                description: "CLI passed config file does not exist.".to_owned(),
                debug: debuginfo!(),
                example: None,
                note: vec![
                    format!("config file '{}' does not exist.", config.to_string_lossy())
                ]
            }
        }
        
        let canonical_config = match config.canonicalize() {
            Ok(canonical_config) => canonical_config,
            Err(err) => {
                return error!{
                    debug: debuginfo!(),
                    description: "could not convert path to a canonical path.".to_owned(),
                    example: None,
                    note: function_message!("std::env::current_dir()", err.to_string())
                }
            }
        };
        
        return Ok((Some(canonical_config), warnings))
        
    }
    
    let extensions = match ConfigLanguage::get_all_extensions() {

        Ok(extensions) => extensions,
        Err(err) => return Err(debugpush!(err))

    }.join(",");

    //                                 example: config.cesty.yaml
    let config_glob: String = format!("*.{CONFIG_FOLLOWUP_NAME}.{{{extensions}}}");

    let mut pwd = match std::env::current_dir() {
        Ok(pwd) => pwd,
        Err(err) => {return error!{
            debug: debuginfo!(),
            description: "could not find the current pwd".to_owned(),
            example: None,
            note: function_message!("std::env::current_dir()", err.to_string())
        }}
    };

    
    loop {
        
        let mut current_directory = pwd.clone();
        current_directory.push(PathBuf::from(&config_glob));

        let max_depth = get_max_depth(&current_directory);

        // TODO:
        // This is garbage, as it requires a conversion from PathBuf -> &str which
        // could fail for some operating systems (as it says in [PathBuf::to_str])
        // glob_builder itself converts &str -> PathBuf so this is just a
        // unnecessary conversion that could fail.
        let walker = match glob_builder(
            current_directory.to_str().expect("Conversion from `PathBuf` into `&str` failed.")
        )
            .case_insensitive(false)
            .contents_first(true)
            .file_type(FileType::FILE)
            .max_depth(max_depth)
            .follow_links(false)
            .build()
        {
            Ok(gw) => gw,
            Err(globerr) => {
                return error!{
                    description: "issue occurred while trying to find a config file".to_owned(),
                    debug: debuginfo!(),
                    example: None,
                    note: function_message!("glowalk::glob_builder().build()", globerr.to_string())
                }
            }
        };


        let file_iter: Vec<Result<DirEntry, WalkError>> = walker.collect();

        if file_iter.len() > 1 {
            return error!{
                debug: debuginfo!(),
                description: format!("found more than one config file in `{}`", current_directory.to_string_lossy().to_string()),
                example: None,
                note: vec![
                    format!("found {} config files", file_iter.len()),
                    "issue occurred while traversing directories in search of a config file".to_owned(),
                    "remove any excess config files from the directory or create a new one more nearer to the pwd".to_owned()
                ]
            }
        }

        for result in file_iter {

            let entry = match result {

                Ok(entry) => entry,
                Err(err) => {
                    warnings.push(warning!{
                        description: "a error was returned while traversing folders in search of a config file".to_owned(),
                        debug: debuginfo!(),
                        example: None,
                        note: vec![
                            "the error message returned is the following:".to_owned(),
                            err.to_string()
                        ]
                    });
                    continue;
                }

            };

            return Ok((Some(entry.path().to_path_buf()), warnings));
        
        }

        if pwd.pop() == false 
        || recursive == false {
            break
        }
        

    }

    Ok((None, warnings))

}

impl ConfigLanguage {

    /// Returns the contained [`InitConfigOptions`].
    pub fn get_options(&self) -> &InitConfigOptions {
        match self {
            ConfigLanguage::TOML(ref options) => options,
            ConfigLanguage::YAML(ref options) => options,
        }
    }

    /// Returns all file extension from the enum variant of
    /// [`ConfigLanguage`] 
    /// 
    /// Note
    /// ----
    /// Only the file extension part is returned without the _"."_
    /// ```
    /// assert_eq(ConfigLanguage::get_extension().unwrap()[0], "toml".to_owned())
    /// ```
    pub fn get_extensions(&self) -> Result<Vec<&'static str>, Alert> {
        
        match self.get_str("extensions") {
            
            Some(string) => Ok(
                string
                    .split_whitespace()
                    .filter(|x| !x.is_empty())
                    .collect::<Vec<&str>>()
            ),
            None => error!{
                debug: debuginfo!(),
                description: "missing `extensions` property for ConfigType".to_string(),
                example: None,
                note: vec![
                    format!("ConfigType::{} does not have an `extension` property", self),
                    "this is a developer issue, try using some other markup language instead or publish a issue on the github page".to_owned(),
                    format!("the supported markup languages for the config are: {:?}", ConfigLanguage::iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")),
                ]
            }      
                
        }
            
    }
    
    /// Returns all file extension from every enum variant of
    /// [`ConfigLanguage`] 
    /// 
    /// Note
    /// ----
    /// Only the file extension part is returned without the _"."_
    pub fn get_all_extensions() -> Result<Vec<&'static str>, Alert> {
    
        let mut extensions: Vec<&'static str> = vec![];
        for conf_type in ConfigLanguage::iter() {
    
            match conf_type.get_extensions() {
                Ok(mut found_extensions) => extensions.append(&mut found_extensions),
                Err(err) => {return Err(debugpush!(err))}
            }

        }
        Ok(extensions)
        
    }

}

impl TryFrom<&OsStr> for ConfigLanguage {

    type Error = Alert;

    fn try_from(value: &OsStr) -> Result<Self, Self::Error> {

        for conf_type in ConfigLanguage::iter() {

            match conf_type.get_extensions() {
                Ok(extensions) => {
                    for extension in extensions {
                        if extension == value {
                            return Ok(conf_type)
                        }
                    }
                }
                Err(err) => return Err(debugpush!(err))
            }

        } 

        return error!{
            debug: debuginfo!(),
            description: format!("failed to find a appropriate ConfigLanguage for extension `{}`", value.to_string_lossy().to_string()),
            example: None,
            note: vec![
                format!("the markup language with the file extension `{}` might not be supported", value.to_string_lossy().to_string()),
                format!("the supported markup languages for the config are: {:?}", ConfigLanguage::iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", "))
            ]
        }
    }

}

impl Config {

    /// Parse the CLI and config file into a [Configuration] struct.
    /// 
    /// Warnings are returned along side the [Configuration] while parsing.
    ///
    /// Otherwise returns a [Alert] error.
    pub fn parse_cli_and_file() -> Result<(Config, Vec<Alert>), Alert> {

        let mut warnings: Vec<Alert> = vec![];

        let cli = Config::parse();
        match cli.command.clone() {
            // Parses both the config and cli.
            Commands::Run(run_conf) => {

                let full_run_conf = match run_conf.reinit() {
                    Ok((initialized, mut ret_warnings)) => {
                        warnings.append(&mut ret_warnings);
                        initialized
                    }
                    Err(err) => return Err(debugpush!(err))
                };

                Ok((Config {
                    command: Commands::Run(full_run_conf)
                }, warnings))

            }
            Commands::Init(init_conf) => {

                Ok((Config {
                    command: Commands::Init(init_conf)
                }, warnings))

            },
        }

    }

}

impl Run {

    /// Re-Initializes a [Run] config by finding and parsing
    /// the file config and merging it into the CLI parsed 
    /// config (presumed to be [self]).
    /// 
    /// Finding the config
    /// ------------------
    /// The config is attempted to be found with [crate::arg_conf::find_config] 
    /// in the current folder and all the parent folders.
    fn reinit(mut self) -> Result<(Self, Vec<Alert>), Alert> {

        let mut warnings: Vec<Alert> = vec![];

        // Order of these 2 is important as .cleanup() reads
        // the current the current directory and 
        // .apply_pre_config_options() changes the current
        // directory.
        // .cleanup is always first.
        self.cleanup()?;
        self.apply_pre_config_options()?;

        let schrodinger_path = match find_config(self.config_path, true) {
            Ok(mut path_and_warnings) => {
                warnings.append(&mut path_and_warnings.1);
                self.config_path = path_and_warnings.0.clone();
                path_and_warnings.0
            },
            Err(err) => return Err(err)
        };

        let config_path = match schrodinger_path {
            Some(path) => {
                let mut config_directory = path.clone();
                config_directory.pop();
                match env::set_current_dir(config_directory) {
                    Ok(_) => {},
                    Err(err) => {
                        return error!{
                            debug: debuginfo!(),
                            description: format!("unable to change the pwd into `{}`", self.directory.as_ref().unwrap().to_string_lossy().to_string()),
                            example: None,
                            note: function_message!("std::env::set_current_dir()", err.to_string())
                        }
                    }
                }
                path
            },
            None =>  {
                if self.no_config == false {
                    warnings.push( warning!{
                        debug: debuginfo!(),
                        description: "no config found".to_owned(),
                        example: None,
                        note: vec![
                            format!("no config file was found in the directory `{}` and all its parent directories.", 
                                std::env::current_dir().unwrap_or_default().to_string_lossy()),
                            "if you don't want to use a config specify the -n / --noconfig flag.".to_owned(),
                            "if your config is located in another directory specify the -c / --config flag along with the relative / absolute path.".to_owned()
                        ]
                    })
                }
                return Ok((self, warnings))
            }
        };

        let config_type = match ConfigLanguage::try_from(config_path.extension().expect("Could not extract extension from PathBuf")) {
            Ok(ty) => ty,
            Err(err) => return Err(debugpush!(err))
        };

        let config_string: String = match std::fs::read_to_string(config_path.clone())
        {
            Ok(s) => s,
            Err(err) => return error!{
                debug: debuginfo!(),
                description: format!("failed to open `{}`", config_path.to_string_lossy().to_string()),
                example: None,
                note: function_message!("std::fs::read_to_string()", err.to_string())
            }
        };

        let parsed = match config_type {

            ConfigLanguage::TOML(_) => {
                match toml::from_str(config_string.as_str()) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        return Err(Alert::from_toml(
                            err, 
                            "unable to parse '.toml' config file".to_owned(),
                            config_path, 
                            debuginfo!()
                        ));
                    }
                }
            }
            ConfigLanguage::YAML(_) => {
                match serde_yaml::from_str(config_string.as_str()) {
                    Ok(parsed) => parsed,
                    Err(err) => {
                        return Err(Alert::from_serde_yaml(
                            err, 
                            "unable to parse '.yaml' config file".to_owned(),
                            config_path, 
                            debuginfo!()
                        ));
                    }
                }
            }

        };

        self.config_merge(parsed);

        Ok((self, warnings))

    }

    /// Applies options from a [Run] struct parsed from the CLI
    /// that might effect finding a config file like changing 
    /// the current pwd.
    fn apply_pre_config_options(&self) -> Result<(), Alert>{

        if self.directory.is_some() {
            match env::set_current_dir(self.directory.as_ref().unwrap()) {
                Ok(_) => {},
                Err(err) => {
                    return error!{
                        debug: debuginfo!(),
                        description: format!("unable to change the pwd into `{}`", self.directory.as_ref().unwrap().to_string_lossy().to_string()),
                        example: None,
                        note: function_message!("std::env::set_current_dir()", err.to_string())
                    }
                }
            }
        }

        Ok(())

    }

    /// Cleanup excess useless data, especially from any [Vec] type members 
    /// inside the [Run] struct.
    fn cleanup(&mut self) -> Result<(), Alert> {

        let current_directory = match env::current_dir() {
            Ok(dir) => {dir},
            Err(err) => {
                return error!{
                    debug: debuginfo!(),
                    description: format!("unable to find current pwd"),
                    example: None,
                    note: function_message!("std::env::current_dir()", err.to_string())
                }
            }
        };

        self
            .files
                .retain(|p| p.to_str().is_some_and(
                    |s| !s.is_empty()
                ));

        self.files = self
            .files
                .iter()
                .map(|x| {
                    let mut path = current_directory.clone();
                    path.push(x.clean());
                    path
                })
                .collect();

        if self.compiler.is_some() {

            self
                .compiler.as_mut().unwrap()
                .flags
                    .retain(|s| !s.is_empty());
            self
                .compiler.as_mut().unwrap()
                .libraries
                    .retain(|s| !s.is_empty());

        }

        Ok(())

    }

    /// Merge 2 [Run] structs, presuming that [self] was obtained
    /// from CLI arguments and `config` was read from a config file.
    /// 
    /// [self] usually takes precedence over `config` but `config`
    /// usually has more members that aren't [None].
    fn config_merge(&mut self, config: Self) {

        *self = Self {
            recipe_name: self.recipe_name.to_owned(),
            no_config:   self.no_config,
            directory:   self.directory.to_owned(),
            files:       self.files.to_owned(),

            config_path: self.config_path.to_owned(),
            list_paths:  self.list_paths.to_owned(),

            compiler: 
            if self.compiler.is_some() 
            {
                Some(CompilerConfig {
                    name: 
                        if self.compiler.as_ref().unwrap().name.is_some() 
                        {
                            self.compiler.to_owned().unwrap().name
                        } 
                        else if config.compiler.as_ref().unwrap().name.is_some() 
                        {
                            config.compiler.to_owned().unwrap().name
                        } 
                        else 
                        {
                            None
                        },
                    flags: 
                        if !self.compiler.as_ref().unwrap().flags.is_empty() 
                        {
                            self.compiler.to_owned().unwrap().flags
                        } 
                        else if !config.compiler.as_ref().unwrap().flags.is_empty() 
                        {
                            config.compiler.to_owned().unwrap().flags 
                        } 
                        else 
                        {
                            vec![]
                        },
                    libraries: 
                        if !self.compiler.as_ref().unwrap().flags.is_empty() 
                        {
                            self.compiler.to_owned().unwrap().flags
                        } 
                        else if !config.compiler.as_ref().unwrap().flags.is_empty() 
                        {
                            config.compiler.to_owned().unwrap().flags 
                        } 
                        else 
                        {
                            vec![]
                        }
                })
            } 
            else if config.compiler.as_ref().is_some() 
            {
                config.compiler.to_owned()
            } 
            else 
            {
                None
            },

            recipes: config.recipes
        };

    }

}

/// [serde] deserializer for [str] and [Vec<String>] into [Vec<String>] 
/// used for compiler libraries and flags such that both of the following 
/// are valid configs:
/// 
/// --------------------------------------------------------
/// ```toml
/// [[compiler]]
/// name = "gcc"
/// flags = ["-std=c11", "-Wall"]
/// libraries = ["-lm"]
/// ``` 
/// --------------------------------------------------------
/// & 
/// 
/// --------------------------------------------------------
/// ```toml
/// [[compiler]]
/// name = "gcc"
/// flags = "-std=c11 -Wall"
/// libraries = "-lm"
/// ```
/// --------------------------------------------------------
/// 
/// Note
/// ----
/// Does cleanup of empty [Vec] members.
pub fn serde_tokenize_strings_and_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where D: Deserializer<'de>
{
    struct StringOrVec(PhantomData<Vec<String>>);

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or list of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where E: de::Error
        {
            Ok(value
                .split_whitespace()
                .map(|item| item.to_owned())
                .filter(|item| item.is_empty() == false)
                .collect()
            )
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>, {
            
            let mut vec = Vec::new();

            while let Some(elem) = seq.next_element::<String>()? {
                
                let mut strings = 
                elem
                    .split_whitespace()
                    .filter(|x| !x.is_empty())
                    .map(|x| x.to_string())
                    .collect::<Vec<String>>();
                vec.append(strings.as_mut())
                
            }
        
            Ok(vec)
            
        }
    }

    deserializer.deserialize_any(StringOrVec(PhantomData))
}
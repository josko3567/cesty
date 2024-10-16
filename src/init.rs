//! Covers the capability of initialization of cesty inside
//! a specific directory.

use strum::IntoEnumIterator;

use std::{
    fs::OpenOptions,
    io::Write,
    path::PathBuf
};

use crate::error::{
    debugpush, 
    function_message,
    Alert, AlertInfo, 
    debuginfo, error
};

use crate::arg_conf::{
    ConfigLanguage, find_config
};

use crate::defaults::{
    DEFAULT_CONFIG_FILENAME, 
    CONFIG_FOLLOWUP_NAME, 
    DEFAULT_PRIVATE_DIRECTORY
};

/// Applies anything needed before running the initializer.
fn apply_pre_init_options(init_conf: &ConfigLanguage) -> Result<(), Alert>{

    let options = init_conf.get_options();

    if options.directory.is_some() {
        match std::env::set_current_dir(options.directory.as_ref().unwrap()) {
            Ok(_) => {},
            Err(err) => {
                return error!{
                    debug: debuginfo!(),
                    description: format!("unable to change the pwd into `{}`", options.directory.as_ref().unwrap().to_string_lossy().to_string()),
                    example: None,
                    note: function_message!("std::env::set_current_dir()", err.to_string())
                }
            }
        }
    }

    Ok(())

}


/// Run the initializer, the executable finishes up after this.
pub fn init(init_conf: ConfigLanguage) -> Result<Vec<Alert>, Alert> {

    let warnings: Vec<Alert> = vec![];

    let options = init_conf.get_options();

    let extension = match init_conf.get_extensions() {
        Ok(extensions) => if extensions.first().is_none() {
            return error!{
                debug: debuginfo!(),
                description: "empty `extensions` property for ConfigType".to_string(),
                example: None,
                note: vec![
                    format!("ConfigType::{} has a `extension` property that is empty", &init_conf),
                    "this is a developer issue, try initializing some other markup language instead".to_owned(),
                    format!("the supported markup languages for the config are: {:?}", ConfigLanguage::iter().map(|x| x.to_string()).collect::<Vec<String>>().join(", ")),
                ]
            }
        } else {
            extensions.first().unwrap().to_owned()
        },
        Err(err) => return Err(debugpush!(err))
    };

    let config_filename: String = if options.name.is_some() {

        options.name.clone().unwrap() 

    } else {

        DEFAULT_CONFIG_FILENAME.to_string() 

    } + "." + CONFIG_FOLLOWUP_NAME + "." + extension;

    let default_config_contents: crate::arg_conf::Run = crate::arg_conf::Run {

        recipe_name: None,
        config_path: None,
        directory:   None,

        no_config:  false,
        list_paths: false,

        files: vec![],

        compiler: Some(crate::arg_conf::CompilerConfig {

            name: Some("gcc".to_owned()),
            flags: vec![
                "-std=c11".to_owned(), 
                "-Wall".to_owned()
            ],
            libraries: vec![
                "-lm".to_owned()
            ]

        }),

        recipes: vec![

            crate::arg_conf::Recipe {

                name: "all".to_owned(),
                force: Some(true),
                prerun: vec![
                    "make".to_owned(), 
                    "meson".to_owned(), 
                    "ninja".to_owned()
                ],
                parse_path: vec![
                    crate::arg_conf::ParsePath {
                        path: PathBuf::from("."),
                        recursive: Some(true)
                    }
                ]

            }

        ]

    };

    match apply_pre_init_options(&init_conf) {
        Ok(_) => (),
        Err(err) => return Err(debugpush!(err))
    };

    match find_config(None, false) {
        Ok(schrodingers_config) => match schrodingers_config.0 {
            Some(path) => {
                if path.is_file() 
                && options.force == true 
                {
                    match std::fs::remove_file(path.clone()) 
                    {
                        Ok(_) => (),
                        Err(err) => return error!{
                            debug: debuginfo!(),
                            description: format!("failed to remove config file `{}`", path.clone().to_string_lossy().to_string()),
                            example: None,
                            note: function_message!("std::fs::remove_file()", err.to_string())
                        }
                    }
                } 
                else
                {
                    return error!{
                        debug: debuginfo!(),
                        description: "cannot initialize twice inside the same directory".to_owned(),
                        example: None,
                        note: vec![
                            "a cesty config already exists in this folder".to_owned(),
                            "use the -f / --force flag to forcibly clean up the folder and overwrite the current config with a default one".to_owned()
                        ]
                    }
                }
            }
            None => ()
        }
        Err(err) => {
            return Err(debugpush!(err))
        }
    };

    
    let mut stream = match {
        // Check if force is on to allow the use of create + truncate otherwise fail
        // if the file exists.
        if options.force == false {
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(config_filename)
        } else {
            OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(config_filename)
        }
    }     
    {
        Ok(stream) => stream,
        Err(err)   => {
            match err.kind() {
                std::io::ErrorKind::AlreadyExists => return error!{
                    debug: debuginfo!(),
                    description: "cannot initialize twice inside the same directory".to_owned(),
                    example: None,
                    note: vec![
                        "a cesty config already exists in this folder".to_owned(),
                        "use the -f / --force flag to forcibly clean up the folder and overwrite the current config with a default one".to_owned()
                    ]
                },

                _ => return error!{
                    debug: debuginfo!(),
                    description: "failed to open/create a new config file".to_owned(),
                    example: None,
                    note: function_message!("std::fs::OpenOptions::new().open()", err.to_string())
                }
            }
        }
    };

    if options.clean {
        match std::fs::remove_dir_all(DEFAULT_PRIVATE_DIRECTORY) {
            Ok(_) => (),
            Err(err) => return error!{
                debug: debuginfo!(),
                description: format!("failed to remove private directory `{}`", DEFAULT_PRIVATE_DIRECTORY),
                example: None,
                note: function_message!("std::fs::remove_dir_all()", err.to_string())
            }
        } 
    }

    match std::fs::create_dir(DEFAULT_PRIVATE_DIRECTORY) {
        Ok(_) => (),
        Err(err) => match err.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => return error!{
                debug: debuginfo!(),
                description: format!("failed to create private directory `{}`", DEFAULT_PRIVATE_DIRECTORY),
                example: None,
                note: function_message!("std::fs::create_dir()", err.to_string())
            }
        }
    }

    let translated_config_contents = match init_conf {

        ConfigLanguage::TOML(_) => {
            match toml::to_string(&default_config_contents){
                Ok(res) => res,
                Err(err) => return error!{
                    debug: debuginfo!(),
                    description: "failed to convert from a structure to TOML".to_owned(),
                    example: None,
                    note: function_message!("toml::to_string()", err.to_string())
                }
            }
        }
        ConfigLanguage::YAML(_) => {
            match serde_yaml::to_string(&default_config_contents) {
                Ok(res) => res,
                Err(err) => return error!{
                    debug: debuginfo!(),
                    description: "failed to convert from a structure to YAML".to_owned(),
                    example: None,
                    note: function_message!("serde_yaml::to_string()", err.to_string())
                }
            }
        }

    };

    match stream.write(translated_config_contents.as_bytes()) {
        Ok(_) => Ok(warnings),
        Err(err) => error!{
            debug: debuginfo!(),
            description: "failed to open/create a new config file".to_owned(),
            example: None,
            note: function_message!("stream.write()", err.to_string())
        }
    }
    
}
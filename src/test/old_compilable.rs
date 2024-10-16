use std::{hash::Hash, path::PathBuf, time::SystemTime};
use rand::Rng;

use crate::{
    defaults::{DEFAULT_PRIVATE_DIRECTORY, MAX_BATCH_ROOT_NAME_CREATION_ATTEMPTS}, 
    error::{
        debuginfo, error, 
        Alert, AlertInfo
    }
};
use super::{Config, extract::ParsedFile};

#[derive(Debug)]
pub struct CompiledTest {

    pub path: PathBuf,
    pub name: String

}

/// What folder is the current test batch going to be
/// placed inside of?
/// 
/// Use
/// ---
/// Creating a test batch folder with:
/// ```
/// batch_folder = TestBatchFolder::new(&run_conf);
/// ```
/// & compiling tests into the folder with:
/// ```
/// let Ok(compiled_test) = batch_folder.compile_into_self(parsed_file)?;
/// ```
/// 
/// No config was found
/// -------------------
/// If no config file exists, instead of creating a `.cesty` folder
/// inside of the folder where `config.cesty.{toml, yaml}` resides,
/// it will create the folder inside of [`std::env::temp_dir()`].
/// 
/// Drop trait
/// ----------
/// Upon finishing, the [Drop] trait of [TestBatchFolder] will spawn
/// a `finish.cesty.lock` file inside of the test batch folder.
#[derive(Debug)]
pub struct TestBatchFolder {

    /// Root of the test folder...
    path: PathBuf,

    /// If a config file doesn't exist, we create a 
    /// [DEFAULT_PRIVATE_DIRECTORY] inside of [std::env::temp_dir].
    path_inside_temp: bool,

}

impl TestBatchFolder {

    pub fn new(config: &crate::arg_conf::Run) -> Result<Self, Alert> {
        
        let (cesty_root, batch_folder_inside_temp) = if config.config_path.is_some() {

            let mut config_root: PathBuf = config.config_path.clone().unwrap(); 
            ({config_root.pop(); config_root}, false)

        } else {

            (std::env::temp_dir(), true)

        };

        
        let batch_folder = {

            let partial_batch_folder = cesty_root.join(DEFAULT_PRIVATE_DIRECTORY);
            let mut batch_folder = PathBuf::new();

            for attempt in 0..=MAX_BATCH_ROOT_NAME_CREATION_ATTEMPTS {

                batch_folder = partial_batch_folder.join(
                    name_from_local_time() + "-" + attempt.to_string().as_str());

                if !batch_folder.exists() {
                    break;
                }

                if attempt == MAX_BATCH_ROOT_NAME_CREATION_ATTEMPTS {
                    return error!{
                        debug: debuginfo!(),
                        description: "failed to create a unique directory name after 10 attempts for this test batch.".to_owned(),
                        note: vec![
                            "This could be a 1/1000000 failure or...".to_owned(),
                            "maybe you are running cesty in parallel inside the same directory?".to_owned()
                        ],
                        example: None
                    }
                }

            }
            batch_folder
        };


        Ok(Self {

            path:             batch_folder,
            path_inside_temp: batch_folder_inside_temp,

        })

    }

    pub fn compile_into_self(&mut self, parsed_file: ParsedFile) -> Result<(CompiledTest, Vec<Alert>), Alert> {

        Err(Alert::default())
        
    }

}



fn name_from_local_time() -> String { // Stolen :P

    let utc = time::OffsetDateTime::UNIX_EPOCH
        + time::Duration::try_from(
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap()
        ).unwrap();

    let local = utc.to_offset(time::UtcOffset::local_offset_at(utc).unwrap());

    format!("{}_{:0>2}_{:0>2}-{:0>2}_{:0>2}_{:0>4}", 
        local.year(), 
        local.month() as u8, 
        local.day(),
        local.hour(),
        local.minute(), 
        local.millisecond()
    )

}
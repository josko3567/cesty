use crate::{
    error::ErrorPosition,
    filegroup::FileGroup,
    globals::{GLOBALS, self},
    config::Config,
    extract::{Extract, ExtractYAML, Test, ExtractYAMLTest},
    environment::Environment,
};

use std::{
    fmt::Display,
    path::{Path, PathBuf}, ffi::{OsString, OsStr}, fs,
    fs::File, io::{ErrorKind, Write}
};
use rand::{distributions::Alphanumeric, Rng};

use colored::{Colorize, ColoredString};
use indoc::formatdoc;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Error {
    CannotCreateFile(ErrorPosition, String),
    FailedFileOperation(ErrorPosition, String, String),
    CannotObtainPWD(ErrorPosition, String),
    CannotCreateFolders(ErrorPosition),
    CannotRemoveFiles(ErrorPosition)
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
            Self::CannotCreateFile(pos, msg) => {
                fmtperr!(pos,
                    "Cannot open file!",
                    "
                        {} failed with the following error...
                            Error: {}
                    ",
                        fmterr_func!(std::fs::File::options()),
                        msg.bright_blue().bold()
                )}
            Self::FailedFileOperation(pos, func, msg) => {
                    fmtperr!(pos,
                        "File operation failed!",
                        "
                            {} failed with the following error...
                                Error: {}
                        ",
                            fmterr_func!(std::fs::File::options()),
                            msg.bright_blue().bold()
                    )}
            Self::CannotObtainPWD(pos, err) => {
                fmtperr!(pos,
                    "Cannot obtain PWD!",
                    "
                        {} failed with this error...
                            Error: {}
                    ",
                        fmterr_func!(std::env::current_dir()),
                        err.bright_blue().bold()
                )}
            Self::CannotCreateFolders(pos) => {
                fmtperr!(pos,
                    "Cannot create test path!",
                    "
                        Failed to create test path...
                            {}
                        ...in PWD.
                        Reason for failure is unknown.
                    ",
                        fmterr_val!(globals::TEST_PATH)
                )}
            Self::CannotRemoveFiles(pos) => {
                fmtperr!(pos,
                    "Cannot delete files!",
                    "
                        Failed to delete files in test path...
                            {}
                        ...due to insufficient privileges!y
                    ",
                        fmterr_val!(globals::TEST_PATH)
                )}
        };
        write!(f, "{message}")
    }
}



impl std::error::Error for Error {}

#[derive(Default)]
pub struct RunnableTest {

    pub name: String,
    pub exec: String,
    pub path: PathBuf,

}

mod picker {

    use std::path::{PathBuf, Path};

    use path_clean::PathClean;

    use crate::{
        config::Config,
        extract::{self, ExtractYAML, Extract},
    };

    pub fn prerun(

        recipe:  Option<String>,
        config:  &Config,
        extract: &ExtractYAML

    ) -> String {

        let mut result = String::new();

        if recipe.is_some()
        && config.recipe.as_ref().is_some() {

            for item in config.recipe.as_ref().unwrap().iter() {

                if &item.name == recipe.as_ref().unwrap()
                && item.prerun.as_ref().is_some() {

                    result += format!("{}; ", item.prerun.as_ref().unwrap().replace('\n', " ")).as_str();
                    break;

                }

            }

        }
        if extract.prerun.is_some() {
        
            extract.prerun
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|s| result += format!("{}; ", s.replace('\n', " ")).as_str())
        
        }

        result

    }

    pub fn compiler_name(

        config:  &Config,
        extract: &ExtractYAML

    ) -> String {

        let mut result = String::new();

        if config.compiler.as_ref().is_some_and(
            |c| c.name.is_some())
        {
    
            result = config.compiler.as_ref().unwrap().name.as_ref().unwrap().clone().replace('\n', " ");
            
        }
        if extract.compiler.as_ref().is_some_and(
            |c| c.name.is_some()
            )
        {
            match extract.compiler.as_ref().unwrap().name.as_ref().unwrap() {
    
                extract::ExtractYAMLCompilerOption::Truncate(new) => {
                    
                    result = new.clone();
    
                }
                extract::ExtractYAMLCompilerOption::Append(new) => {
    
                    if new.append == true {
    
                        result += new.new.clone().replace('\n', " ").as_str();
    
                    } else if new.append == false {
    
                        result = new.new.clone().replace('\n', " ");
    
                    }
    
                }
    
            }

        }

        return result;

    }

    pub fn compiler_flags(

        config:  &Config,
        extract: &ExtractYAML

    ) -> String {

        let mut result = String::new();

        if config.compiler.as_ref().is_some_and(
            |c| c.flags.is_some())
        {
    
            result = config.compiler.as_ref().unwrap().flags.as_ref().unwrap().clone().replace('\n', " ");
            
        }
        if extract.compiler.as_ref().is_some_and(
            |c| c.flags.is_some()
            )
        {
            match extract.compiler.as_ref().unwrap().flags.as_ref().unwrap() {
    
                extract::ExtractYAMLCompilerOption::Truncate(new) => {
                    
                    result = new.clone();
    
                }
                extract::ExtractYAMLCompilerOption::Append(new) => {
    
                    if new.append == true {
    
                        result += new.new.clone().replace('\n', " ").as_str();
    
                    } else if new.append == false {
    
                        result = new.new.clone().replace('\n', " ");
    
                    }
    
                }
    
            }

        }

        return result;

    }

    pub fn compiler_libraries(

        config:  &Config,
        extract: &ExtractYAML

    ) -> String {

        let mut result = String::new();

        if config.compiler.as_ref().is_some_and(
            |c| c.libraries.is_some())
        {
            
            result = config.compiler.as_ref().unwrap().libraries.as_ref().unwrap().clone().replace('\n', " ");
            
        }
        if extract.compiler.as_ref().is_some_and(
            |c| c.libraries.is_some()
            )
        {
            match extract.compiler.as_ref().unwrap().libraries.as_ref().unwrap() {
    
                extract::ExtractYAMLCompilerOption::Truncate(new) => {
                    
                    result = new.clone();
    
                }
                extract::ExtractYAMLCompilerOption::Append(new) => {
    
                    if new.append == true {
    
                        result += new.new.clone().replace('\n', " ").as_str();
    
                    } else if new.append == false {
    
                        result = new.new.clone().replace('\n', " ");
    
                    }
    
                }
    
            }

        }

        return result;

    }

    pub (super) fn include_to_string(v: &Vec<String>, extract: &Extract) -> String {

        let mut culmination = String::new();
        
        for include in v.iter() {

            culmination += format!("#include {}\n", {
                let locality = |inc: &str|{
                    if inc.starts_with("<")
                    && inc.ends_with(">")
                    {
                        return inc.to_owned()
                    }
                    else if inc.starts_with("\"")
                    &&      inc.ends_with("\"")
                    {
                        return inc.to_owned()
                    }
                    else 
                    {
                        if PathBuf::from(inc).is_absolute() {
                            return format!("\"{inc}\"")
                        } else {
                            let inc_absolute = {
                                let mut tmp = PathBuf::from(extract.filepath.clone());
                                tmp.pop();
                                tmp.push(inc);
                                tmp.clean()
                            };
                            return format!("\"{}\"", inc_absolute.to_string_lossy().to_string())
                        }
                    }
                };
                locality(include.trim())  
            }).as_str()

        }

        culmination

    }

}

fn matching_filename(function_name: &String) -> OsString {

    let parts = function_name.split_once('(');
    let name = if parts.is_some() {
        parts.unwrap().0
    } else {
        function_name
    };

    Path::new(&format!("{}_f{}",
        name,
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect::<String>()
    )).as_os_str().to_os_string()

}

fn get_testy_path(

    config: &Config

) -> Result<PathBuf, Error> 
{

    let mut tmp = if config.path.is_none() {
        match std::env::current_dir() {
            Ok(res) => {res.to_path_buf()}
            Err(err) => {reterr!(Error::CannotObtainPWD, err.to_string())}
        }
    }
    else {
        let mut a = PathBuf::from(config.path.as_ref().unwrap());
        a.pop();
        a
    };
    tmp.push(globals::TEST_PATH);
    return Ok(tmp);

}

pub fn create_testy_path(

    config: &Config

) -> Result<(), Error>
{

    match get_testy_path(config) {
        Ok(testypath) => {
            if fs::create_dir_all(&testypath).is_err() {
                reterr!(self::Error::CannotCreateFolders)
            } else {
                Ok(())
            }
        }
        Err(err) => {
            Err(err)
        }
    }

}

pub fn remove_testy_path(

    config: &Config

) -> Result<(), Error> 
{

    match get_testy_path(config) {
        Ok(testypath) => {
            if fs::remove_dir_all(&testypath).is_err_and(
                |err|{err.kind() == ErrorKind::PermissionDenied}
            ) {
                reterr!(self::Error::CannotRemoveFiles);
            } else {
                Ok(())
            } 
        }
        Err(err) => {
            Err(err)
        }
    }

}

fn build_file_contents(
    extract:           &Extract,
    test:              &Test,
    extract_yaml:      &ExtractYAML,
    extract_yaml_test: &ExtractYAMLTest,
    environment:       &Environment
) -> String 
{

    let test_func = "__cesty_test_".to_owned() + {let lastpart = ||{
        if extract_yaml_test.name.is_some() {
            if extract_yaml_test.name.as_ref().unwrap().contains(
                |c: char|{
                    if c.is_alphanumeric() || c == '_' {false} else {true}
                }
            ) {
                "noname"
            } else {
                extract_yaml_test.name.as_ref().unwrap().as_str()
            }
        } else {
            "noname"
        }
    }; lastpart()};

    let used_env = if extract_yaml.info.as_ref().is_some_and(
        |info| {info.standalone.is_some_and(
            |standalone| standalone == true
        )}
    ) {
        &environment.full
    } else {
        &environment.clean
    };

    let ret_main = if extract_yaml_test.expect == true {
        format!("return {test_func}(argc, argv) ? 0 : 1;")
    } else {
        format!("return {test_func}(argc, argv) ? 1 : 0;")
    };
    
    let cesty_inc = if extract_yaml.include.is_some() {
        picker::include_to_string(&extract_yaml.include.as_ref().unwrap(), extract)
    } else {
        String::new()
    };


    return formatdoc!("
        // #! CESTY INCLUSIONS.
        #include <stdbool.h>
        {cesty_inc}
        // #! FILE ENVIRONMENT
        {used_env}
        // #! CESTY TEST FUNCTION
        bool {test_func}(int argc, char ** argv) {{
            {}
        }}
        // #! CESTY MAIN FUNCTION
        int main(int argc, char ** argv) {{
            {ret_main}
        }}        
    ",
        extract_yaml_test.code.replace("\n", "\n\t").trim_end()
    );

}

pub fn build_test(

    recipe:            Option<String>,
    config:            &Config,
    extract:           &Extract,
    test:              &Test,
    extract_yaml:      &ExtractYAML,
    extract_yaml_test: &ExtractYAMLTest,
    environment:       &Environment,
    subname:           &String

) -> Result<Option<RunnableTest>, Error>
{

    // Check if we can run the builder with this m o n s t e r.
    if extract_yaml.info.as_ref().is_some_and(
        |info| info.run.is_some_and(
            |run| run == false
        )
    ) 
    && recipe.is_some()
    && config.recipe.as_ref().is_some_and(
        |recipes| recipes.iter().find(
            |item| &item.name == recipe.as_ref().unwrap()
        ).is_some_and(
            |found| found.force.is_some_and(
                |force| force == false))
    )
    {
        return Ok(None)
    }

    // let mut runtest: RunnableTest = RunnableTest::default();
    
    let prerun = picker::prerun(recipe, config, extract_yaml);

    let compiler_name:      String = picker::compiler_name(config, extract_yaml);
    let compiler_flags:     String = picker::compiler_flags(config, extract_yaml);
    let compiler_libraries: String = picker::compiler_libraries(config, extract_yaml);
    
    let filename = Path::new(&matching_filename(&test.function)).to_owned();

    let testypath = match get_testy_path(config) {
        Ok(res) => {res}
        Err(err) => {return Err(err)}
    };

    let cfile  = {
        let mut tmp = testypath.clone(); 
        tmp.push(&filename);
        tmp.set_extension("c"); 
        tmp
    };
    
    let exfile = {
        let mut tmp = testypath.clone(); 
        tmp.push(&filename);
        tmp.set_extension({
            let ext = ||{
                if std::env::consts::OS == "windows" {
                    OsStr::new("exe")
                } else {
                    OsStr::new("out")
                }};
            ext()
        }); 
        tmp
    };

    let mut stream = match File::options()
        .create_new(true)
        .write(true)
        .read(false)
        .truncate(true)
        .open(&cfile)
    {
        Ok(file) => {file}
        Err(err) => {
            reterr!(
                self::Error::CannotCreateFile, 
                err.to_string()
            )
        }
    };

    match stream.write_all(
        build_file_contents(
            extract, 
            test, 
            extract_yaml, 
            extract_yaml_test, 
            environment
        ).as_bytes()
    )
    {
        Ok(_) => {}
        Err(err) => {
            reterr!(
                self::Error::FailedFileOperation, 
                fmterr_func!(stream.write_all(build_file_contents())),
                err.to_string()
            )
        }
    }

    let testname = if extract_yaml_test.name.is_some() {
        extract_yaml_test.name.as_ref().unwrap().to_owned()
    } else {
        subname.to_owned()                
    };

    let exec = format!("{}{} {} {:?} -o {:?} {}", 
        prerun, 
        compiler_name,
        compiler_flags,
        cfile,
        exfile,
        compiler_libraries
    );

    Ok(Some(
        RunnableTest {
            name: testname,
            path: exfile,
            exec: exec
        }
    ))
    
}
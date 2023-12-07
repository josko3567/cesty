use crate::{
    error::ErrorPosition,
    filegroup::FileGroup,
    globals::GLOBALS,
    config::Config,
    extract::{Extract, self, ExtractYAML},
    environment::Environment, argument::Argument,
};

use std::{
    fmt::Display,
    path::Path
};

use colored::Colorize;
use indoc::formatdoc;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Error {
    CannotCreateFile(ErrorPosition)
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
            Self::CannotCreateFile(pos) => {
                if GLOBALS.read().unwrap().get_warn() { fmtpwarn!(pos,
                "No config file was found.",
                "
                    Proceeding with argument passed files.
                ")}
                else {
                    "".white()
                }}
        };
        write!(f, "{message}")
    }
}



impl std::error::Error for Error {}

#[derive(Default)]
pub struct RunnableTest {

    pub name: String,
    pub path: String,
    pub exec: String

}

mod picker {

    use crate::{
        config::Config,
        extract::{Extract, self, ExtractYAML},
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

}

/* HOW A FILE SHOULD LOOK

/// {prerun} {compiler.name} {compiler.flags} file.c -o file.out {compiler.libraries} 
// ENVIRONMENT
#include <stdio.h>
...
// MAIN
int main(int argc, char ** argv)
{

    bool result = false;

    code...

    return result ? 0 : 1;
    or
    return !result ? 0 : 1;
    
}


*/
pub fn build_test(

    recipe:       Option<String>,
    config:       &Config,
    extract:      &Extract,
    extract_yaml: &ExtractYAML,
    environment:  &Environment,

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

    let mut runtest: RunnableTest = RunnableTest::default();
    
    runtest.exec = picker::prerun(recipe, config, extract_yaml);

    let compiler_name:      String = picker::compiler_name(config, extract_yaml);
    let compiler_flags:     String = picker::compiler_flags(config, extract_yaml);
    let compiler_libraries: String = picker::compiler_libraries(config, extract_yaml);

    

    println!("{} {} {} CUM.c -o POOP.out {}", 
        runtest.exec, 
        compiler_name,
        compiler_flags,
        compiler_libraries
    );

    // let compiler_flags: String = picker

        // Compiler flags
    // let mut compiler_flags

    Ok(Some(runtest))

}
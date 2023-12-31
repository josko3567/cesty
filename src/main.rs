#[macro_use]
mod error;
use builder::RunnableTest;
#[allow(unused_imports)]
use error::*;
#[allow(unused_imports)]
use colored::Colorize;
use globals::AccessLevel;
#[allow(unused_imports)]
use std::path::Path;
#[allow(unused_imports)]
use indoc::formatdoc;
#[allow(unused_imports)]
use indoc::indoc;

#[allow(unused_imports)]
use core::panic;
#[allow(unused_imports)]
use std::{env, process::exit};

// -----------------------
// Files from this project
#[macro_use]
mod filegroup;

mod globals;
#[allow(unused_imports)]
use crate::globals::GLOBALS;

#[allow(dead_code)]
mod argument;
use crate::argument::Argument;

#[allow(dead_code)]
mod config;
use crate::config::Config;

#[allow(dead_code)]
mod lister;

#[allow(dead_code)]
mod clang;
#[allow(unused_imports)]
use clang_sys::*;
use crate::clang::*;

#[allow(dead_code)]
mod extract;
use crate::extract::*;

#[allow(dead_code)]
mod environment;
use crate::environment::*;

mod builder;


pub fn cesty(argument: &Vec<String>) -> Result<(), String> {


    // return Ok(());

    let mut args = match 
        Argument::try_from_vec(argument) 
    {

        Ok(res) => {res}
        Err(err) => { 
                eprintln!("{}", err);
                return Err(err.code())
        }

    };

    
    
    let arg_recipe = args.iter().find(|a|{
        match a {
            Argument::Recipe(_) => {true}
            _ => {false}
        }
    });
    
    let recipe = if arg_recipe.is_some() {
        match arg_recipe.unwrap() {
            Argument::Recipe(name) => {Some(name.clone())}
            _ => {Some("".to_string())}
        }
    } else {
        None
    };
    
    let mut config: Config = Config::new();
    
    match config.from_file(config::find()) {
        
        Err(err) => { 
            
            match err {
                config::Error::NoConfigFile(_) => {
                    // eprintln!("{:?}", GLOBALS.read().unwrap().get_noconfig_al());
                    if GLOBALS.read().unwrap().get_warn() 
                    && GLOBALS.read().unwrap().get_noconfig()
                    && GLOBALS.read().unwrap().get_noconfig_al() == AccessLevel::Config {
                        eprintln!("{}", err)
                    }
                }
                _ => {
                    eprintln!("{}", err);
                    return Err(err.code())
                }
            }
            
        }
        _ => {}
        
    }
    
    config.merge_overrides(&args);
    
    
    let files = 
    match lister::get_list(&config, &args) {
        Ok(list) => {list},
        Err(err) => {
            eprintln!("{err}"); 
            return Err(err.code());
        }
    };
    
    match builder::remove_testy_path(&config) {
        Err(err) => {
            eprintln!("{err}");
            return Err(err.code())
        }
        _ => {}
    }

    match builder::create_testy_path(&config) {
        Err(err) => {
            eprintln!("{err}");
            return Err(err.code())
        }
        _ => {}
    }

    for file in files {

        let clang = match Clang::from_lister(&file) {
            Ok(res) => {res}
            Err(err) => {
                eprintln!("{}", err);
                return Err(err.code());
            }
        };            

        let extract = Extract::from_lister(
            &file, 
            clang.cur.clone()
        );

        let environment = match Environment::from_lister(
            &file, 
            clang.cur.clone())
        {
            Ok(res) => {res},
            Err(err) => {
                eprintln!("{}", err);
                return Err(err.code());
            }
        };

        // println!("--------- >>>");
        // println!("{}", environment.full);
        // println!("\t<<< ---------");
        // println!("{}", environment.clean);

        let mut runnable_tests: Vec<RunnableTest> = vec![];

        for test 
        in extract.tests.iter() {

            for extract_yaml 
            in test.yaml.iter() {

                if extract_yaml.test.is_some() {
                    
                    for extract_yaml_test 
                    in extract_yaml.test.as_ref().unwrap().iter().enumerate() {

                        let subname = 
                            test.function.clone() 
                            + "_#" 
                            + extract_yaml_test.0.to_string().as_str();
    
                        match builder::build_test(
                            recipe.clone(), 
                            &config,
                            &extract, 
                            test,
                            extract_yaml, 
                            extract_yaml_test.1, 
                            &environment,
                            &subname
                        ) {
        
                            Ok(opt) => {
                                match opt {
                                    Some(res) => {
                                        runnable_tests.push(res)
                                    }
                                    _ => {}
                                }
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                                return Err(err.code());
                            }
        
                        }

                    }

                }

            }

        }

        clang.close();

    }

    Ok(())

}

#[allow(unused_variables)]
fn main() -> Result<(), String> {

    #[cfg(debug_assertions)]
        env::set_var("RUST_BACKTRACE", "full");

    let res = cesty(&env::args().skip(1).collect());
    println!("{}", env::current_dir().unwrap().as_path().to_str().unwrap());
    res

}
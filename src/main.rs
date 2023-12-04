#[macro_use]
mod error;
#[allow(unused_imports)]
use error::*;
#[allow(unused_imports)]
use colored::Colorize;
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

#[allow(dead_code)]
mod translate;


#[allow(unused_variables)]
fn main() -> Result<(), String> {

    #[cfg(debug_assertions)]
    env::set_var("RUST_BACKTRACE", "full");

    let mut args = match 
        Argument::try_from_vec(&env::args().collect()) 
    {

        Ok(res) => {res}
        Err(err) => { 
                eprintln!("{}", err);
                return Err(err.code())
        }

    };

    let mut conf: Config = Config::new();

    match conf.from_file(config::find()) {

        Err(err) => { 
            
            eprintln!("{}", err);
            match err {
                config::Error::NoConfigFile(_) => {}
                _ => {return Err(err.code())}
            }
        
        }
        _ => {}
        
    }
    
    conf.merge_overrides(&args);

    println!("{:#?}", conf);
    // args.iter().for_each(|x| x.print());
    
    let files = 
    match lister::get_list(&conf, &args) {
        Ok(list) => {list},
        Err(err) => {
            eprintln!("{err}"); 
            return Err(err.code());
        }
    };

    for file in files {

        let clang = match Clang::from_lister(&file) {
            Ok(res) => {res}
            Err(err) => {
                eprintln!("{}", err);
                return Err(err.code());
            }
        };
            

        let res = Extract::from_lister(
            &file, 
            clang.cur.clone()
        );
        // {
        //     Ok(res) => {res}
        //     Err(err) => { match err {
        //         ExtractError::NothingToExtract(file) => {
        //             Extract::default()
        //         },
        //         _ => {
        //             eprintln!("{}", err);
        //             return Err(err.code());
        //         }
        //     }}
        // };



        println!("{:#?}:\n",  
            &res.filepath
        );

        res.tests.iter().for_each(|x| {
            println!(
                "{} {}\n\n{}:{}",
                x.returns,
                x.function,
                x.line,
                x.column
            );
            x.yaml.iter().for_each(|x| println!(
                "---\n\
                {:#?}\
                ...\n",
                x
            ))
        });

        let env = match Environment::from_lister_into_pool(
            &file, 
            clang.cur.clone())
        {
            Ok(res) => {res},
            Err(err) => {
                eprintln!("{}", err);
                return Err(err.code());
            }
        };
        
        clang.close();

    }

    println!("{}", GLOBALS.read().unwrap().get_message_amount());
    
    Ok(())

}
mod error;

#[allow(unused_imports)]
use core::panic;
#[allow(unused_imports)]
use std::{env, process::exit};

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

    let mut args = match 
        Argument::from_vec(&env::args().collect()) 
    {

        Ok(res) => {res}
        Err(err) => { match err {

            argument::Error::UnknownArgument(ref str) => {
                eprintln!("{}", err);
                return Err(err.code())
            },
            _ => { return Err(err.code()) }

        }}

    };
    
    let mut conf: Config = Config::new();

    match conf.from_file(config::find_config()) {

        Err(err) => { match err {

            config::ConfigError::NoConfigFile => {
                eprintln!("{}", err)
            },
           _ => {
                eprintln!("{}", err);
                return Err(err.code());
            }
        
        }}
        _ => {}
        
    }
    
    conf.merge_overrides(&args);

    // Append only non existent options
    match Argument::from_string_(
        conf.cesty.as_ref().unwrap().flags.as_ref()) 
    {
        Ok(vector) => {
            let mut filtered: Vec<Argument> = vector.into_iter().filter(
                |x| args.iter().find(|y| &x == y).is_none()
            ).collect();
            args.append(&mut filtered);
        },
        Err(err) => { match err {

            argument::Error::NoString => {},
            _ => {
                return Err(err.code());
            }

        }}
    }    

    // println!("{:#?}", conf);
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
            

        let res = 
        match Extract::from_lister(
            &file, 
            clang.cur.clone()
        ) 
        {
            Ok(res) => {res}
            Err(err) => { match err {
                ExtractError::NothingToExtract(file) => {
                    Extract::default()
                },
                _ => {
                    eprintln!("{}", err);
                    return Err(err.code());
                }
            }}
        };

        println!("{:#?}:\n",  
            &res.filepath
        );

        // res.tests.iter().for_each(|x| {
        //     println!(
        //         "{} {}\n\n{}:{}",
        //         x.returns,
        //         x.function,
        //         x.line,
        //         x.column
        //     );
        //     x.comment.iter().for_each(|x| println!(
        //         "---\n\
        //         {}\
        //         ...\n",
        //         x
        //     ))
        // });

        let env = 
        match Environment::from_lister(
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
    
    Ok(())

}
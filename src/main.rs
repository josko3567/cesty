//! Cesty is a testing command line utility for C.
//! 
//! Unlike most testing utilities for C, cesty isn't a
//! library to be included in your code for running tests.
//! 
//! Cesty via. its configuration file scans the source code
//! for function with the prefix `cesty_` and runs them,
//! listing if they failed or not (like a test 0_o).
//! 
//! Due to this, for more complex projects the library/binary
//! must be compiled prior to the test and the resulting 
//! library/object files must be passed to the test function 
//! via. individualized test configurations found in the function 
//! documentation (or global configuration from the config file).
//! 
//! A config file can be initialized with `cesty init <MARKUP LANGUAGE>`.
//! 
//! Currently in the config file YAML & TOML are allowed markup languages
//! whilst in the individual configs from function documentation only
//! TOML is allowed (due to it being friendly for non highlighted markup).
//! 
//! What directories/files to parse for tests can be written
//! in individual recipes found inside the configuration file.
//! 
//! More files to parse can also be passed via. the command line.
//! 
//! For every recipe you can configure the name, files scanned,
//! compiler settings & give a list of commands to run
//! such that the code is compiled before the tests
//! are ran (if needed).
//! 
//! Tests are ran with `cesty run <RECIPE>`
//! 
//! For more information about all commands and options
//! for the command line interface please use `cesty --help`,
//! `cesty run --help` & `cesty init --help`

mod arg_conf;
mod error;
mod init;
mod defaults;
mod lister;
mod rustclang;
mod test;


pub fn cesty(conf: arg_conf::Config) -> Result<(), Box<dyn std::error::Error>> {

    let run_conf = match conf.command {
        arg_conf::Commands::Run(run_conf) => run_conf,
        arg_conf::Commands::Init(init_conf) => match init::init(init_conf) {
            Ok(res) => {
                for warning in res {eprintln!("{warning}")}
                return Ok(())
            },
            Err(err) => {
                eprintln!("{err}");
                return Err(Box::new(err))
            }
        }
    };

    let (list, _recipe) = match lister::list(&run_conf) {
        Ok(((list, recipe), warnings)) => {
            for warning in warnings {eprintln!("{warning}")}
            (list, recipe)
        }
        Err(err) => {
            eprintln!("{err}");
            return Err(Box::new(err))
        } 
    };

    if run_conf.list_paths == true {

        for path in list {
            println!("{}", path.to_string_lossy())
        }
        return Ok(())
        
    }

    for path in list {

        let parsed_file = match test::extract::extract(path){
            Ok((parsed_file, warnings)) => {
                for warning in warnings {eprintln!("{warning}")}
                parsed_file
            },
            Err(err) => {
                eprintln!("{err}");
                return Err(Box::new(err));
            }
        };

        if parsed_file.test.is_empty() == true {
            continue
        }

        println!("Found test inside of {:?}", &parsed_file.path);

        // for test in parsed_file.test.iter() {
        //     println!("{:?}", test.get_test_file_stem(&parsed_file));
        //     println!("{:#?}", test);
        // }

        println!("{:#?}", parsed_file);


    }

    return Ok(())

} 

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let conf: arg_conf::Config = match arg_conf::Config::parse_cli_and_file() {
        Ok(conf) => {
            for warning in conf.1 {eprintln!("{warning}")}
            conf.0
        },
        Err(err) => {
            eprintln!("{err}");
            return Err(Box::new(err))
        }
    };

    cesty(conf)

}

#[cfg(test)]
mod tests {

    use crate::error::{self, debuginfo};

    #[test]
    fn error_printout() {
        let err = error::Alert::Error( error::AlertInfo {
            description: "Lorem ipsum dolor sit amet.". to_owned(),
            debug: debuginfo!(),
            example: Some( error::AlertExample::Code( error::AlertCode{
                file: "config.yaml.cesty".to_owned(),
                line: 14,
                code: "let lorem = error::Error::default();".to_owned(),
                fix: vec![
                    error::AlertCodeFix{
                        relative_line: 0,
                        column: 6,
                        comment: "`lorem` is already taken, use something else.".to_owned()
                    },
                    error::AlertCodeFix{
                        relative_line: 0,
                        column: 6,
                        comment: "`lorem` is already taken, use something else.".to_owned()
                    },
                ]
            })),
            note: vec!["heelo".to_owned(), "hi".to_owned()]
    
        });
        println!("+");
        println!("{err}");
        println!("+");
    }

    #[test]
    fn find_config_test() {

        println!("{:#?}", crate::arg_conf::find_config(true));

    }
}

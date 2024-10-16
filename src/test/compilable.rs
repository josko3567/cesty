//! Functions for creating a compilable test.
//!
//! These functions both create the directories and files.
//! Where they are created depend on the circumstance:
//! - If there is no config file found, the files/directories
//!   get created in the temporary folder.
//! - If there is a config file found, the location of where the
//!   tests end up depend on argument/config variables.
//!   - If "-t/--temp" is set in the config file or inside the arguments
//!     the current test will be created inside the temporary folder.
//!   - Otherwise the test is created inside the ".cesty" folder located
//!     in the same directory as the config file.
//!
//! ./cesty run all -D ../..

use std::path::PathBuf;

use indoc::{formatdoc, indoc};

use super::extract::{ParsedFile, ParsedTest};

/// A file created from a [super::extract::ParsedTest].
pub struct CompilableTest {

    /// Parsed from the function docs.
    pub config: super::Config,

    /// File path of the compilable test.
    pub path: PathBuf

}

fn create_compilable_test(parsed_test: &ParsedTest, parsed_file: &ParsedFile) -> String {
    
    formatdoc!{
        "
            {env}
            
            int main() {{
                
                return {func}() == true ? 0 : 1;
                
            }}
        ",
        env  = parsed_file.environment.mainless,
        func = parsed_test.function.name
    }
    
}

impl CompilableTest {

    pub fn from_parsed_file(
        parsed_file: ParsedFile,
        config: &crate::arg_conf::Run
    ) -> Vec<CompilableTest> {
        
        let compilable_tests: Vec<CompilableTest> = vec![];
        
        for parsed_test in parsed_file.test.iter() {
            
            let file = create_compilable_test(&parsed_test, &parsed_file);
            
            println!("{}", file);
            
            
        }
        
        compilable_tests
        
    }
    
    
}

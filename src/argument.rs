use crate::error::ErrorGroup;

use std::{
    // error::Error, 
    fmt::Display, 
    path::Path
};

use indoc::indoc;
use colored::Colorize;
use strum_macros::*;

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Error {
    RenamedExecutable(String),
    NoArguments,

    UnknownArgument(String),
    UnknownOverrideNoEqual(String),
    UnknownOverrideNoDashes(String),
    NoString
    
}

impl Error {

    pub fn code(&self) -> String {
        return format!("E;{:X}:{:X}", 
            ErrorGroup::from(
                Path::new(file!())
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap()
            ) as u8, 
            // Took this off of some stack overflow thread, 
            // converts self (ArgumentError) into a enumerated number.
            unsafe { *(self as *const Self as *const u8) }
        );
    }
    
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::RenamedExecutable(name) => {
                format!( indoc!{"
                Error!
                    hey... did you rename MY program to {}?!?!?11
                    What a god awful name...
                    Change it back to cesty NOW O - O
                    I'm counting on you, ill make your PC explode if you don't OKEY!??!?
                "}, name.red().bold())
                .red()},
            Self::NoArguments => {
                format!( indoc!{"
                Error!
                    0 arguments were provided, not even the executable name?
                "})
                .red()},
            Self::UnknownArgument(name) => {
                format!( indoc!{"
                Error! 
                    Unknown argument:
                        {}
                "}, name.red().bold())
                .red()},
            Self::UnknownOverrideNoEqual(name) => {
                format!( indoc!{"
                Error! 
                    Probable override:
                        Override: {}
                    Is missing a '=' sign.
                "}, name.red().bold())
                .red()},
            Self::UnknownOverrideNoDashes(name) => {
                format!( indoc!{"
                Error! 
                    Probable override:
                        Override: {}
                    Is missing 2 dashes at the start.
                "}, name.red().bold())
                .red()},
            Self::NoString => {
                "".normal()}
        };
        write!(f, "{}\n{message}", 
            format!("From {}...", 
                Path::new(file!())
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap()
                .bold()
            ).dimmed()
        )
    }
}

impl std::error::Error for Error {}

#[derive(EnumProperty, EnumIter)]
#[derive(PartialEq)]
#[repr(u32)]
pub enum Override {

    #[strum(props(
        ignore         ="false",
        full           ="",
        start          ="",
        seperate_name  =".",
        seperate_value ="=",

        first  ="", 
        second ="", 
        third  ="", 
        forth  =""
    ))]
    Unknown((String, String)),

    #[strum(props(
        ignore         ="false",
        full           ="--compiler.name=",
        start          ="--",
        seperate_name  =".",
        seperate_value ="=",

        first  ="compiler", 
        second ="name", 
        third  ="", 
        forth  =""
    ))]
    CompilerName(String),

    #[strum(props(
        ignore         ="false",
        full           ="--compiler.flags=",
        start          ="--",
        seperate_name  =".",
        seperate_value ="=",
        
        first  ="compiler", 
        second ="flags", 
        third  ="", 
        forth  =""
    ))]
    CompilerFlags(String),

    #[strum(props(
        ignore         ="false",
        full           ="--compiler.libraries=",
        start          ="--",
        seperate_name  =".",
        seperate_value ="=",

        first  ="compiler", 
        second ="libraries", 
        third  ="", 
        forth  =""
    ))]
    CompilerLibraries(String)
}

impl Default for Override {
    fn default() -> Self {
        return Override::Unknown((String::new(), String::new()))
    }
}

#[derive(EnumProperty, EnumIter)]
#[derive(PartialEq)]
#[repr(u32)]
pub enum Argument {

    #[strum(props(
        neighbor_input ="0",
        rest_as_input  ="false",
        full           ="",
        start          ="",
        seperate_name  ="",
        seperate_value ="",
    ))]
    Unknown((String, String)),

    #[strum(props(
        // -1 Uses remaining strings in vec as input
        neighbor_input ="-1",
        rest_as_input  ="false",
        full           ="-f", // Matches -f, returns rest of vec as input.
        start          ="-",
        seperate_name  ="",
        seperate_value ="",
    ))]    
    Files(Vec<String>),

    #[strum(props(
        neighbor_input ="-1",
        // Stores the rest of the string as input
        rest_as_input  ="true",
        full           ="-m", // Matches -m, returns rest as input.
        start          ="-",
        seperate_name  ="",
        seperate_value ="",
    ))]
    MessageAmount(String),

    #[strum(props(
        neighbor_input ="-1",
        rest_as_input  ="true",
        // Uses a specific position in the input
        specific_pos   ="1",
        full           ="", // Matches nothing, and returns rest as input.
        start          ="",
        seperate_name  ="",
        seperate_value ="",
    ))]
    Recipe(String),

    // The very existence of the prop ignore makes it ignore this value
    #[strum(props(ignore="true"))]
    Override(Override)
}

impl Default for Argument {
    fn default() -> Self {
        return Argument::Unknown((String::new(), String::new()))
    }
}

impl Argument {

    fn value() -> u32 {

    }

}

// impl Argument {

    // pub fn is_maybe(s: &str) -> bool {

    //     return if  s.starts_with("-") 
    //     && !s.starts_with("--") {
    //         true
    //     } else {
    //         false
    //     }

    // }

    // pub fn from_string_( str: Option<&String> ) -> Result<Vec<Argument>, Error> {
        
    //     let Some(pure) = str else {
    //         return Err(Error::NoString);
    //     };

    //     Argument::from_vec(
    //         &pure
    //             .split_whitespace()
    //             .map(String::from)
    //             .collect()
    //     )

    // }



    // pub fn from_vec( vector: &Vec<String> ) -> Result<Vec<Argument>, Error> {
    
    //     let mut res: Vec<Argument> = Vec::new();

    //     // Check for renamed exec.
    //     let Some(zeroarg) = vector.first() else {
    //         return Err(Error::NoArguments);
    //     };

    //     let executable =
    //     match Path::new(zeroarg).file_name() 
    //     {
    //         Some(res) => {res},
    //         None => {return Err(Error::NoArguments)}
    //     }
    //     .to_string_lossy()
    //     .to_string();

    //     if executable != "cesty" {
    //         return Err(Error::RenamedExecutable(executable));
    //     } 
        
    //     let mut iter = vector.into_iter().skip(1);
    //     let firstarg = vector.get(1);

    //     loop {
    
    //         let Some(str) = iter.next() else {
    //             break;
    //         };
    
    //         let split = str
    //             .split_once('=');
    
    //         let sidekick = match split {
    //             Some(split) => {split.1.to_string()}
    //             None                      => {String::from("")}
    //         };
    
    //         let purecmd = match split {
    //             Some(split) => {split.0.to_string()}
    //             None => {str.clone()}
    //         };
            
    //         match purecmd.as_str() {
    //             "-ml" => {res.push(Argument::MessageAmount('l'));}
    //             "-mm" => {res.push(Argument::MessageAmount('m'));}
    //             "-ms" => {res.push(Argument::MessageAmount('s'));}
    //             "--compiler.name" => {
    //                 res.push(Argument::OverrideOptions(
    //                         Override::CompilerName(sidekick)
    //                     )
    //                 );
    //             }
    //             "--compiler.flags" => {
    //                 res.push(Argument::OverrideOptions(
    //                         Override::CompilerFlags(sidekick)
    //                     )
    //                 );
    //             }
    //             "--compiler.libraries" => {
    //                 res.push(Argument::OverrideOptions(
    //                         Override::CompilerLibraries(sidekick)
    //                     )
    //                 );
    //             }
    //             "-f" => {
    //                 let mut files = Vec::<String>::new();
    //                 loop {
    //                     let Some(file) = iter.next() else {
    //                         break;
    //                     };
    //                     files.push(file.to_string());
    //                 }
    //                 res.push(Argument::Files(files));
    //             }
    //             _ => {
    //                 if firstarg.is_some_and(|first| first == &purecmd) 
    //                 && !purecmd.contains(['-', '=', '/'].as_ref()) {

    //                     res.push(Argument::Recipe(str.to_string()));

    //                 } else {

    //                     return Err(Error::UnknownArgument(purecmd));

    //                 }
    //             }
    //         }
    
    //         // i+=1;
    
    //     }
    
    //     return Ok(res);
        
    // }

    // pub fn print(&self) {
    //     match &self {
    //         Argument::Files(files) => {
    //             println!("Argument::Files =>");
    //             for file in files.into_iter() {
    //                 println!("\t{}",file);
    //             }
    //         }
    //         Argument::MessageAmount(amount) => {
    //             println!("Argument::MessageAmount => {}",
    //                 amount
    //             );
    //         }
    //         Argument::Recipe(recipe) => {
    //             println!("Argument::Recipe => {}",
    //                 recipe
    //             );
    //         }
    //         Argument::OverrideOptions(option) => {
    //             print!("Argument::OverrideOption => ");
    //             match option {
    //                 OverrideOption::CompilerName(name) => {
    //                     println!("--compiler.name = {}", name)
    //                 }
    //                 OverrideOption::CompilerLibraries(name) => {
    //                     println!("--compiler.libraries = {}", name)
    //                 }
    //                 OverrideOption::CompilerFlags(name) => {
    //                     println!("--compiler.flags = {}", name)
    //                 }
    //             }
    //         }
    //     }
    // }

// }

// pub fn new<T>(

//     value: T

// ) -> Result<Argument, ArgumentError>
// {

//     if 

// }

// pub fn From(

//     s: String

// ) -> Result<OverrideOption, ArgumentError> 
// {

//     // String::from(value)

//     if s.is_empty() {
//         return Err(ArgumentError::UnknownArgument(s))
//     }

//     let strcln = s.trim();

//     //
//     if Argument::probably(strcln) {

//         let argument = Argument::new()
        
//     } else if OverrideOption::probably(strcln) {



//     }

//     Err(ArgumentError::NoArguments)

// }


impl TryFrom<&str> for Argument {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        if value.is_empty() {
            return Err(Self::Error::UnknownArgument(String::from(value)));
        }

        return Err(Error::UnknownArgument(String::new()));

    }

}
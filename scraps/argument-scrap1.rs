positionuse crate::error::ErrorGroup;

use std::{
    // error::Error, 
    fmt::Display, 
    path::Path, sync::Mutex, collections::HashMap
};

use indoc::indoc;
use colored::Colorize;
use itertools::Itertools;
use lazy_static::lazy_static;
use strum::{IntoEnumIterator, EnumProperty};
use strum_macros::*;

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Error {
    RenamedExecutable(String),
    NoArguments,

    InvalidPropertyValue((String, String, String)),
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
            Self::InvalidPropertyValue((name, property, reason)) => {
                format!( indoc!{"
                Cesty Error! 
                    The property of a Argument/Override:
                        Argument/Override: {}
                    Has property \"{}\" that has a invalid value for the following reason:
                        Reason: {}
                "}, 
                    name.red().bold(),
                    property.red().bold(),
                    reason.red().bold()
                )
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
#[derive(PartialEq, Debug, Clone)]
#[repr(u32)]
/// Override a compiler value for in all tests (excluding untouchables).
/// All overrides start with two dashes ''--''
/// # Examples
/// Here
/// ```bash
/// cesty all --compiler.name=clang
/// ```
/// All tests are whose values are aren't untouchable have their compiler
/// change from gcc to clang.
pub enum Override {

    #[strum(props(ignore="true"))]
    Unknown((String, String)),

    #[strum(props(
        body            ="--compiler.name=",
        slice_in        ="true",
        start           ="--",
        body_separator  =".",
    ))]
    CompilerName(String),

    #[strum(props(
        body            ="--compiler.flags=",
        slice_in        ="true",
        start           ="--",
        body_separator  =".",
    ))]
    CompilerFlags(String),

    #[strum(props(
        body            ="--compiler.libraries=",
        slice_in        ="true",
        start           ="--",
        body_separator  =".",
    ))]
    CompilerLibraries(String)
     
}

impl std::fmt::Display for Override {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Override {
    fn default() -> Self {
        return Override::Unknown((String::new(), String::new()))
    }
}

impl Override {

    pub(super) fn to_key(&self) -> String {
        if self.get_str("position")format!("")
    }

    pub(super) fn value(&self) -> u32 {
        unsafe { *(self as *const Self as *const u32) }
    }

}

#[derive(EnumProperty, EnumIter)]
#[derive(PartialEq, Debug, Clone)]
#[repr(u32)]
/// Standard cesty flags and arguments.
/// All of them are listed in the -h command with their specifics.
/// # Rules
/// There are a "few" set of rules when it comes to argument bodies.
/// - start of the "body" must be equal to "start"
/// - "start", "value_separator" & "name_separator" must be a non alphanumeric value.
/// - everything else in "body" must be a alphanumeric value
/// with the exception of the characters of "value_separator" & "name_separator".
/// - "value_separator" must be at the end and it muse be a '=' if you are using it.
/// - "body" must be unique.
/// - if the "body" is empty so must be "start".
/// - if the "body" is empty, "specific_pos" must be specified (aka non zero).
/// - if the "body" is empty unless "slice_in" is set to true we wont take the 
/// string at the specific position as input.
pub enum Argument {

    #[strum(props(ignore="true"))]
    Unknown(String),

    #[strum(props(
        neighbor_in    ="-1", // -1 Uses remaining strings in vec as input
        slice_in       ="false",
        position       ="0", // 0 is supposed to be the executable, so this means that there is no specific pos
        body           ="-f", // Matches -f, returns rest of vec as input.
        start          ="-"
    ))]    
    Files(Vec<String>),

    #[strum(props(
        neighbor_in    ="0", // Equal to not placing the value.
        slice_in       ="true", // Stores the rest of the string as input, aka the slice
        values         ="l, m, s", // Available values to match.
        position       ="0",
        body           ="-m", // Matches -m, returns rest as input if it matches a entry from values.
        start          ="-"
    ))]
    MessageAmount(String),

    #[strum(props(
        neighbor_in    ="0",
        slice_in       ="true", // Uses a specific position in the input
        position       ="1", // First argument is the recipe, -1 would be for last argument.
        body           ="", // Matches nothing at pos 1, and returns rest as input.
        start          =""
    ))]
    Recipe(String),

    // The very existence of the prop ignore makes it ignore this value
    #[strum(props(ignore="true"))]
    Overrides(Override)

}

impl std::fmt::Display for Argument {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for Argument {
    fn default() -> Self {
        return Argument::Unknown(String::new())
    }
}

impl Argument {

    pub(super) fn to_key(&self) -> String {
        format!("A.{}", self.to_string())
    }

    pub(super) fn value(&self) -> u32 {
        unsafe { *(self as *const Self as *const u32) }
    }

}


#[derive(Debug,Clone)]
pub(super) struct Property {

    name  :  (&'static str, String),
    body  :  (&'static str, String),
    start : (&'static str, String),
    position: (&'static str, i32),

    // body_separator : (&'static str, Option<char>),
    value_separator: Option<char>,
    values: (&'static str, Vec<String>),

    neighbor_in : (&'static str,  i32),
    slice_in    : bool,

    // parts:  Vec<String>,
    argument: Argument

}

impl Default for Property {

    fn default() -> Self {
        return Self { 

            name:     ("name",  String::new()),
            body:     ("body",  String::new()),
            start:    ("start", String::new()), 
            position: ("position", 0), 

            value_separator: None,
            values: ("values", Vec::new()), 
            
            slice_in:     false, 
            neighbor_in:  ("neighbor_in",  0), 
            
            argument: Argument::default()

            // body_separator:  ("body_separator",  None), 
            // parts:  Vec::new(),
            
            
        }
    }

}

pub(super) fn type_name<T>(_: &T) -> String { 
    format!("{}", std::any::type_name::<T>())
}


impl Property {

    // Cant impl TryFrom as there is already a blanket impl in core.
    fn generic_try_from<T: strum::EnumProperty + std::fmt::Display> (
        value: &T,
    ) -> Result<Self, self::Error> {
        let mut property = Property::default();

    property.name.1 = value.to_string();

    property.body.1 = 
        if value
            .get_str(property.body.0)
            .is_some()
        {
            String::from(value.get_str(property.body.0).unwrap())
        } 
        else 
        {
            return Err(self::Error::InvalidPropertyValue((
                value.to_string(),
                String::from(property.body.0),
                String::from("You buffon, you didn't make a \
                value for the Property!")
            )));  
        };


    property.start.1 = 
        if value
            .get_str(property.start.0)
            .is_some() 
        {
            String::from(value.get_str(property.start.0).unwrap())
        }
        else 
        {
            return Err(self::Error::InvalidPropertyValue((
                value.to_string(),
                String::from(property.start.0),
                String::from("You buffon, you didn't make a \
                value for the Property!")
            )));  
        };


    // property.body_separator.1 = 
    //     if value
    //         .get_str(property.body_separator.0)
    //         .is_some_and(|x| x.len() == 1) 
    //     {
    //         let sep = value
    //             .get_str(property.body_separator.0)
    //         .unwrap();
    //         Some(sep.chars().next().unwrap())
    //     } 
    //     else if value
    //         .get_str(property.body_separator.0)
    //         .is_none() 
    //     {
    //         Property::default().body_separator.1
    //     }
    //     else 
    //     {
    //         return Err(self::Error::InvalidPropertyValue((
    //             value.to_string(), 
    //             String::from(property.body_separator.0),
    //             String::from("You buffon, you put multiple characters \
    //             in the Property. There can be only one!")
    //         )));
    //     };


    if property.body.1.trim_end().chars().last().is_some_and(|x| x == '=') 
    {

        property.value_separator = Some('=')

    } else {

        property.value_separator = None;

    }
    // property.value_separator.1 = 
    //     if value
    //         .get_str(property.value_separator.0)
    //         .is_some_and(|x| x.len() == 1) 
    //     {
    //         let sep = value
    //             .get_str(property.value_separator.0)
    //         .unwrap();
    //         Some(sep.chars().next().unwrap())
    //     } 
    //     else if value
    //         .get_str(property.value_separator.0)
    //         .is_none() 
    //     {
    //         Property::default().value_separator.1
    //     }
    //     else 
    //     {
    //         return Err(self::Error::InvalidPropertyValue((
    //             value.to_string(), 
    //             String::from(property.value_separator.0),
    //             String::from("You buffon, you put multiple characters \
    //             in the Property. There can be only one!")
    //         )));
    //     };


    property.position.1 = 
        if value
            .get_str(property.position.0)
            .is_some_and(|x| x.parse::<i32>().is_ok() ) 
        {
            value
                .get_str(property.position.0)
                .unwrap()
                .parse()
            .unwrap()
        } 
        else if value
            .get_str(property.position.0)
            .is_none() 
        {
            Property::default().position.1
        }
        else 
        {
            return Err(self::Error::InvalidPropertyValue((
                value.to_string(), 
                String::from(property.position.0),
                String::from(format!(
                "You buffon, you didn't put a {} value as the Property \
                value!",
                    type_name(&property.position.1)
                ))
            )));
        };


    property.neighbor_in.1 = 
        if value
            .get_str(property.neighbor_in.0)
            .is_some_and(|x| x.parse::<i32>().is_ok() ) 
        {
            value
                .get_str(property.neighbor_in.0)
                .unwrap()
                .parse()
            .unwrap()
        } 
        else if value
            .get_str(property.neighbor_in.0)
            .is_none() 
        {
            Property::default().neighbor_in.1
        }
        else 
        {
            return Err(self::Error::InvalidPropertyValue((
                value.to_string(), 
                String::from(property.neighbor_in.0),
                String::from(format!(
                "You buffon, you didn't put a {} value as the Property \
                value!",
                    type_name(&property.neighbor_in.1)
                ))
            )));
        };
    

    // property.slice_in.1 = 
        // if value
        //     .get_str(property.slice_in.0)
        //     .is_some_and(|x| x.parse::<bool>().is_ok() ) 
        // {
        //     value
        //         .get_str(property.slice_in.0)
        //         .unwrap()
        //         .parse()
        //     .unwrap()
        // } 
        // else if value
        //     .get_str(property.slice_in.0)
        //     .is_none() 
        // {
        //     Property::default().slice_in.1
        // }
        // else 
        // {
        //     return Err(self::Error::InvalidPropertyValue((
        //         value.to_string(), 
        //         String::from(property.slice_in.0),
        //         String::from(format!(
        //         "You buffon, you didn't put a {} value as the Property \
        //         value!",
        //             type_name(&property.slice_in.1)
        //         ))
        //     )));
        // };


    property.values.1 =
        if value
            .get_str(property.values.0)
            .is_some_and(|x| !x.is_empty() ) 
        {
            value
                .get_str(property.values.0)
                .unwrap()
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|x| String::from(x.trim()))
            .collect()
        } 
        else if value
            .get_str(property.values.0)
            .is_none() 
        {
            Property::default().values.1
        }
        else 
        {
            return Err(self::Error::InvalidPropertyValue((
                value.to_string(), 
                String::from(property.values.0),
                String::from(format!(
                "You buffon, you didn't put a {} value as the Property \
                value!",
                    type_name(&property.values.1)
                ))
            )));
        };

    property.slice_in = 
        if !property.values.1.is_empty() {
            true
        }
        else {
            false
        };

    // This part checks if the string itself is valid by cesty
    // argument standards
    // property.parts = 
    //     if property.body.1.is_empty()
    //     && property.start.1.is_empty() 
    //     {
    //         if property.position.1 == 0
    //         {
    //             return Err(self::Error::InvalidPropertyValue((
    //                 value.to_string(), 
    //                 String::from(property.body.0),
    //                 String::from(format!(
    //                 "You buffon, your \"full\" & \"start\" are both \
    //                 empty yet your specific_pos is 0. Set a specific
    //                 position or this argument will match everything!"
    //                 ))
    //             )));
    //         } 
    //         else
    //         {
    //             Vec::new()
    //         }
    //     }
    //     else
    //     {

    //         if property.body.1
    //             .chars()
    //             .find(|x| x.is_whitespace())
    //             .is_some()
    //         {
    //             return Err(self::Error::InvalidPropertyValue((
    //                 value.to_string(), 
    //                 String::from(property.body.0),
    //                 String::from(format!(
    //                 "You buffon, you whitespaces in this thing, what \
    //                 is wrong with you?!",
    //                 ))
    //             )));
    //         }

    //         if !property.body.1
    //             .starts_with(&property.start.1)
    //         || property.start.1.len() >= property.body.1.len()
    //         {
    //             return Err(self::Error::InvalidPropertyValue((
    //                 value.to_string(), 
    //                 String::from(property.body.0),
    //                 String::from(format!(
    //                 "You buffon, your \"argument\" only contains the \
    //                 the start portion \"{}\"!",
    //                 &property.start.1
    //                 ))
    //             )));
    //         }

    //         let ch = property.body.1
    //             .split_at(property.start.1.len())
    //             .1
    //             .chars();

    //         let invalid_ch =
    //             ch.clone()
    //             .into_iter()
    //             .find_position(|x| {
    //                 if !x.is_alphanumeric()
    //                 && !property.body_separator.1
    //                     .is_some_and(|y| x == &y)
    //                 && !property.value_separator
    //                     .is_some_and(|y| x == &y)
    //                 {
    //                     true
    //                 } 
    //                 else 
    //                 {
    //                     false
    //                 }
    //             });

    //         if invalid_ch.is_some()
    //         {
    //             return Err(self::Error::InvalidPropertyValue((
    //                 value.to_string(), 
    //                 String::from(property.body.0),
    //                 String::from(format!(
    //                 "You buffon, your \"argument\" without the start \
    //                 portion can only contain alphanumeric values, the \
    //                 name separator symbol or the value separator symbol!",
    //                 ))
    //             )));
    //         }


    //         property.body.1
    //             .as_str()[property.start.1.len()..property.body.1.len()]
    //             .split(|x| 
    //                 property.body_separator.1.is_some_and(
    //                     |y| x == y
    //                 )
    //                 ||
    //                 property.value_separator.is_some_and(
    //                     |y| x == y
    //                 )
    //             )
    //             .map(String::from)
    //             .filter(|x| !x.is_empty())
    //             .collect()
            

    //     };

    return Ok(property);

    }

}

lazy_static!(
    pub(super) static ref PROPERTIES: Mutex<HashMap<String, (Argument, Property)>> 
    = Mutex::new(
        HashMap::new()
    );
);

pub fn print_property_pools() {

    PROPERTIES.lock().unwrap().iter().for_each(|x| println!("{:#?}", x.1));

}

pub fn fill_property_pools() -> Result<(), self::Error> 
{

    for arg in Argument::iter() {

        if arg
            .get_str("ignore")
            .is_some_and(|x| x.to_lowercase() != "false") 
        {
            continue;
        }

        let property =
        match Property::generic_try_from(&arg) {

            Ok(res) => {res},
            Err(err) => {
                return Err(err)
            }

        };

        // if property.values == 
        
        if PROPERTIES
            .lock()
            .unwrap()
            .insert(property.body.1.clone(), (arg.clone(), property.clone()))
            .is_some()
        {

            return Err(self::Error::InvalidPropertyValue((
                arg.to_string(),
                String::from(property.body.0),
                format!("Property must be unique yet Argument {} has the same body!",
                PROPERTIES.lock().unwrap().get(&property.body.1).unwrap().0.to_string()
                )
            )));

        }

    }

    for arg in Override::iter() {

        if arg
            .get_str("ignore")
            .is_some_and(|x| x.to_lowercase() != "false") 
        {
            continue;
        }

        let property =
        match Property::generic_try_from(&arg) {

            Ok(res) => {res},
            Err(err) => {
                return Err(err)
            }

        };
        
        if PROPERTIES
            .lock()
            .unwrap()
            .insert(property.body.1.clone(), (Argument::Overrides(arg.clone()), property.clone()))
            .is_some()
        {

            return Err(self::Error::InvalidPropertyValue((
                arg.to_string(),
                String::from(property.body.0),
                format!("Property must be unique yet Override {} has the same body!",
                    match &PROPERTIES.lock().unwrap().get(&property.body.1).unwrap().0 {
                        Argument::Overrides(res) => {res.to_string()}
                        _ => {arg.to_string()}
                    }
                )
            )));

        }

    } 

    Ok(())

}

impl TryFrom<&str> for Argument {
    type Error = self::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {

        if value.is_empty() {
            return Err(Self::Error::UnknownArgument(String::from(value)));
        }

        if PROPERTIES.lock().unwrap().is_empty() {
            match fill_property_pools() {
                Err(err) => {return Err(err)},
                _ => {}
            }
        }

        for arg in Argument::iter() {


            

        }

        

        let clean = value.trim_start();


        return Err(Error::UnknownArgument(String::new()));

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
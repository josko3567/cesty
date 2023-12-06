use crate::{
    config::Config,
    error::ErrorPosition,
    filegroup::FileGroup,
    globals::{GLOBALS, self}
};

use std::{
    fmt::Display, 
    path::Path, 
    sync::Mutex, 
    collections::HashMap,
    iter::{Enumerate, Peekable}, ops::Deref,
};

use indoc::formatdoc;
use colored::Colorize;
use itertools::Itertools;
use lazy_static::lazy_static;
use strum::{IntoEnumIterator, EnumProperty};
use strum_macros::*;
use strsim::normalized_damerau_levenshtein;


#[repr(u8)]
#[derive(Clone, Debug)]
pub enum Error {
    //                File, func, line...|New bin name
    RenamedExecutable(ErrorPosition, String),
    NoArguments(ErrorPosition),
    //                             Argument name, Similar
    UnknownArgument(ErrorPosition, String, String),
    //                               Arg/Ovr name|Propr name|Reason
    InvalidPropertyValue(ErrorPosition, String, &'static str, String),
}

impl Error {

    pub fn code(&self) -> String {
        return format!("E;{:X}:{:X}", 
            FileGroup::from(
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
            Self::RenamedExecutable(pos, name) => {
                fmtperr!(pos,
                "Renamed executable!",
                "
                    Hey... did you rename M Y program to {}?!?!??!?11111
                    What a GOD awful name...
                    Change it back to cesty... NOW <O - O>
                    I'm counting on you...
                " ,name)}
            Self::NoArguments(pos) => {
                fmtperr!(pos,
                "No arguments found",
                "
                    Zero arguments were provided for cesty, not even the executable name???
                ")}
            Self::UnknownArgument(pos, name, similar) => {
                fmtperr!(pos,
                "Invalid argument!", 
                "
                    Invalid command line argument...
                      {}{}{}
                ", fmterr_val!{name}, 
                    if similar.is_empty() {""} else {"\n"}, 
                    similar)}
            Self::InvalidPropertyValue(pos, name, property, reason) => {
                fmtperr!(pos,
                "Developer mistake, improper EnumProperty value found.",
                "
                    A property of a Argument/Override...
                        {}
                    ...under the alias...
                        {}
                    ...has a invalid value for the following reason:
                        {}
                ", fmterr_val!(name), fmterr_val!(property), reason.bold())}
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

#[derive(EnumProperty, EnumIter)]
#[derive(PartialEq, Debug, Clone)]
#[repr(u32)]
/// Override a compiler value for in all tests (excluding untouchables).
/// All overrides here start with two dashes `--`.
/// # Example
/// ```bash
/// cesty all --compiler.name="clang"
/// ```
/// All tests whose `compiler: name:` value isn't untouchable have their compiler
/// change from whatever they had before to `clang`.
/// # Warning.
/// If you want your override to take affect please handle it in 
/// [`config::Config::merge_overrides`](crate::config::Config)
pub enum Override {

    #[strum(props(ignore="true"))]
    Unknown((String, String, String)),

    #[strum(props(
        body            ="--compiler.name=",
        start           ="--",
        body_separator  =".",
    ))]
    CompilerName(String),

    #[strum(props(
        body            ="--compiler.flags=",
        start           ="--",
        body_separator  =".",
    ))]
    CompilerFlags(String),

    #[strum(props(
        body            ="--compiler.libraries=",
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
        return Override::Unknown((String::new(), String::new(), String::new()))
    }
}

impl Override {

    pub(super) fn value(&self) -> u32 {
        unsafe { *(self as *const Self as *const u32) }
    }

}

/// Standard cesty flags and arguments.
/// All of them are listed in the -h command with their specifics.
/// 
/// # Making a argument.
/// To create a argument:
/// - Create a new value in either structure: [`Argument`] or [`Override`].
/// - Give it [`strum`] [`EnumProperty`] to your liking, how to give proper 
///   properties is explained in the [rules](#rules) section.
/// - Handle it in [`Self::from_parts()`].
/// - Run in debug mode to see if the argument properties have any errors.
/// - Enjoy!
/// 
/// # Argument properties:
/// Given with [`EnumProperty`] like this.
/// ```
/// struct Argument {
///     
///     ...
/// 
///     #[strum(props(
///         ignore          = "false",
///         body            ="--my.arg=",
///         start           ="--",
///         body_separator  =".",
///         position        = "0"
///     ))]
///     MyArgument(String)
/// 
///     ...
/// 
/// }
/// ```
/// 
/// - `body`<br>
///   The argument identifier.
///   The *main body* is `body` without `start` or `value_separator`.
/// 
///   Example, `body` is <ins>underlined</ins> & *main body* is **bold** 
///   (non bold - & bold **-** for reference):
///   - <ins>--<b>compiler.name</b>=</ins>"gcc"
///   - <ins>--<b>compiler.flags</b>=</ins>"-std=c99"
///   - <ins>-<b>f</b></ins>
///   - <ins>-<b>m</b></ins>m
///   - <ins>-<b>m</b></ins>s
/// 
/// - `start`<br>
///   Starting indicator of the argument. Every flag usually has a `-` 
///   or `--` to indicate they are flags. This is only used to check if 
///   the *main body* contains valid characters.
/// 
///   Example:
///   - <ins>--</ins>compiler.name="gcc"
///   - <ins>--</ins>compiler.flags="-std=c99"
///   - <ins>-</ins>f
///   - <ins>-</ins>mm
///   - <ins>-</ins>ms    
/// 
/// - `position`<br>
///   In what position inside the vector of arguments can I find this value?
///   If position is non 0 (aka 1, 2, etc...) the argument doesn't need 
///   `body` or `start` values as it takes the entire string as input.
/// 
///   Example:
///   ```bash
///   cesty all
///   ``` 
///   `all` is a positional argument and its position is in 1.
/// 
///   - __Note__<br>
///     Positional arguments are read second which means that if the argument
///     at some position is an actual argument and not a random string, the 
///     argument will be returned instead of the positional argument.
///
///     Example:
///     ```bash
///     cesty all #all is matched to Argument::Recipe(String::from("all"))
///     cesty -mm #-mm is matched to Argument::MessageAmount(String::from("m"))
///     ```
/// - `value_separator` <em>(unused!)</em><br>
///   `value_separator` isn't a user defined value anymore, instead the global
///   (to this mod) character used for `value_separator` is `=`.
///   To use it just put a `=` at the end of your `body`.
///   It's the equivalent of writing `value_separator="="`.
///   Everything after the separator is taken as input.
///
/// - `body_separator`<br>
///   Contains a non alphanumerical character that is allowed inside the *main body*.
///   
///   Example where `body_separator="."` and *main body* is **bold**:
///   - --<b>compiler<ins>.</ins>name</b>="gcc"
///   - --<b>compiler<ins>.</ins>flags</b>="-std=c99"
/// 
/// - `ignore`<br>
///   Self explanatory, if set to `true` the argument is used for anything.
/// 
/// <a name="rules"></a>
/// # Rules
/// There are a "few" rules when it comes to creating argument properties.
/// - The start of `body` must be equal to `start`
/// - `start` & `body_separator` must contain only (a) non alphanumeric value/s.
/// - The *main body* of `body` (excludes `start` & `value_separator` from the 
///   `body`) must contain only alphanumeric values with the exception of the 
///   character contained in `body_separator`.
/// - The entire `body` must be unique.
/// - Arguments that have predefined values (`values`) must also be unique when
///   that value is given. (Example: if there are 2 arguments `-ma` & `-m`,
///   argument `-m` cannot have a predefined value of `a` since that would
///   collide with the `-ma` argument).
/// - For a argument to accept a user given value (`values`), a `=` must be
///   placed at the end indicating the argument has a `value_separator`.
/// - The `body` cannot be a empty string unless we have `position` set to a
///   non zero number.
/// - If `position` is non zero, the entire string passed as an argument is
///   taken as input. Before that the string is matched to an actual argument
///   and if it corresponds to a existing argument we returned the matched
///   argument instead.
#[derive(EnumProperty, EnumIter)]
#[derive(PartialEq, Debug, Clone)]
#[repr(u32)]
pub enum Argument {
    
    #[strum(props(ignore="true"))]
    Unknown((String, String, String)),

    #[strum(props(
        position       ="0", // 0 is supposed to be the executable, so this means that there is no specific pos
        body           ="-f", // Matches -f, returns rest of vec as input.
        start          ="-"
    ))]    
    Files(Vec<String>),

    #[strum(props(
        values         ="l, m, s", // Available values to match.
        position       ="0",
        body           ="-m", // Matches -m, returns rest as input if it matches a entry from values.
        start          ="-"
    ))]
    MessageAmount(String),

    #[strum(props(
        position       ="1", // First argument is the recipe, -1 would be for last argument.
    ))]
    Recipe(String),

    #[strum(props(
        position       ="0", // First argument is the recipe, -1 would be for last argument.
        body           ="--help", // Matches nothing at pos 1, and returns rest as input.
        start          ="--"
    ))]
    PrintInstructions,

    #[strum(props(
        position       ="0", // First argument is the recipe, -1 would be for last argument.
        body           ="-w", // Matches nothing at pos 1, and returns rest as input.
        start          ="-"
    ))]
    Warnings,

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
        return Argument::Unknown((String::new(), String::new(), String::new()))
    }
}

impl Argument {

    pub(super) fn value(&self) -> u32 {
        unsafe { *(self as *const Self as *const u32) }
    }

}

/// Predefined [`Argument`] & [`Override`] properties to do general
/// argument matching.
/// 
/// Properties for each [`Argument`] & [`Override`] are attached with 
/// [`EnumProperty`](strum::EnumProperty) from [`strum`].
/// 
/// For more detail on what each item does refer to 
/// [`Argument`](self::Argument).
#[derive(Debug,Clone)]
pub(super) struct Property {

    name            : String,
    body            : (&'static str,       String),
    start           : (&'static str,       String),
    position        : (&'static str,          i32),
    values          : (&'static str,  Vec<String>),
    body_separator  : (&'static str, Option<char>),
    value_separator : Option<char>,

}

impl Default for Property {

    fn default() -> Self {
        return Self { 

            name            : String::new(),
            body            : ("body",  String::new()),
            start           : ("start", String::new()), 
            position        : ("position",          0), 
            values          : ("values",   Vec::new()), 
            body_separator  : ("body_separator", None),
            value_separator : None,

        }
    }

}

const VALUE_SEPARATOR_SYMBOL: char = '=';

pub(super) fn type_name<T>(_: &T) -> String { 
    format!("{}", std::any::type_name::<T>())
}

impl Property {

    /// Convert a generic type that implements [`EnumProperty`](strum::EnumProperty) 
    /// & [`Display`](std::fmt::Display) into a [`Property`].
    /// 
    /// # Example
    /// ```
    /// let property = 
    /// match Property::generic_try_from(&Override::CompilerName(String::new()))
    /// {
    ///     Ok(res) => {res}
    ///     Err(err) => {return Err(err)}
    /// }
    /// ```
    /// # Debug
    /// During debug release the code raises errors that document what
    /// went wrong.
    /// Use these errors to fix your custom properties.
    /// 
    /// # Release
    /// A lot of unwraps, so panics are possible only if you didn't run it in
    /// debug before hand and it worked flawlessly.
    pub(super) fn generic_try_from<T: strum::EnumProperty + std::fmt::Display> (
        value: &T,
    ) -> Result<Self, self::Error> {

        let mut property = Property::default();

        property.name = value.to_string();

        #[cfg(debug_assertions)] {
        property.position.1 = 
            if value
                .get_str(property.position.0)
                .is_some_and(|x| x.parse::<i32>().is_ok_and(|x| !x.is_negative()) ) 
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
                reterr!{
                    self::Error::InvalidPropertyValue,
                        value.to_string(), 
                        property.position.0,
                        format!(
                            "You buffon, you didn't put a {} value as the Property \
                            value! That or it's negative or to big.",
                            type_name(&property.position.1)
                        )
                };
            };
        }
        #[cfg(not(debug_assertions))] {
        property.position.1 = 
            match value.get_str(property.position.0) 
            {
                Some(res) => {res.parse::<i32>().unwrap()}
                None => {Property::default().position.1}
            }
        }

        #[cfg(debug_assertions)] {
        property.body.1 = 
            if value
                .get_str(property.body.0)
                .is_some()
            {
                String::from(value.get_str(property.body.0).unwrap())
            } 
            else if property.position.1 > 0
            {
                String::new()
            }
            else 
            {
                reterr!(
                    self::Error::InvalidPropertyValue,
                        value.to_string(),
                        property.body.0,
                        format!(
                            "You buffon, you didn't make a \
                            value for the Property!"
                        )
                );  
            };
        }
        #[cfg(not(debug_assertions))] {
        property.body.1 = match value.get_str(property.body.0){
            Some(res) => {String::from(res)}
            None => {String::new()}
        };}

        #[cfg(debug_assertions)] {
        property.start.1 = 
            if value
                .get_str(property.start.0)
                .is_some() 
            {
                String::from(value.get_str(property.start.0).unwrap())
            }
            else if property.position.1 > 0
            {
                String::new()
            }
            else 
            { 
                reterr!(
                    self::Error::InvalidPropertyValue,
                        value.to_string(),
                        property.start.0,
                        format!(
                            "You buffon, you didn't make a \
                            value for the Property!"
                        )
                );  
            };
        }
        #[cfg(not(debug_assertions))] {
        property.start.1 = match value.get_str(property.start.0){
            Some(res) => {String::from(res)}
            None => {String::new()}
        };}

        property.value_separator =
            if property.body.1
                .trim_end()
                .chars()
                .last()
                .is_some_and(|x| x == VALUE_SEPARATOR_SYMBOL) 
            {
                Some(VALUE_SEPARATOR_SYMBOL)
            } 
            else 
            {
                None
            };
           
        #[cfg(debug_assertions)] {
        property.body_separator.1 = 
            if value
                .get_str(property.body_separator.0)
                .is_some_and(|x| x.len() == 1) 
            {
                let sep = value
                    .get_str(property.body_separator.0)
                    .unwrap();
                Some(sep.chars().next().unwrap_or('.'))
            } 
            else if value
                .get_str(property.body_separator.0)
                .is_none()
            || value
                .get_str(property.body_separator.0)
                .is_some_and(|x| x.is_empty())
            {
                Property::default().body_separator.1
            }
            else 
            {
                reterr!(
                    self::Error::InvalidPropertyValue,
                        value.to_string(),
                        property.body_separator.0,
                        format!(
                            "You buffon, you put multiple characters \
                            in the Property. There can be only one!"
                        )
                );
            };
        }
        #[cfg(not(debug_assertions))] {
        property.body_separator.1 = 
            match value.get_str(property.body_separator.0) 
            {
                Some(res) => {Some(res.chars().next().unwrap_or('.'))}
                None => {Property::default().body_separator.1}
            }
        }

        #[cfg(debug_assertions)] {
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
                reterr!(
                    self::Error::InvalidPropertyValue,
                        value.to_string(), 
                        property.values.0,
                        format!(
                            "You buffon, you didn't put a {} value as the Property \
                            value!",
                            type_name(&property.values.1)
                        )
                );
            };
        }
        #[cfg(not(debug_assertions))] {
        property.values.1 =
            match value.get_str(property.values.0)
            {
                Some(res) => {
                    res
                    .split(',')
                    .filter(|x| !x.is_empty())
                    .map(|x| String::from(x.trim()))
                    .collect()
                }
                None => {Property::default().values.1}
            }
        }


        // This part checks if the string itself is valid by cesty
        // argument standards.
        #[cfg(debug_assertions)]
        if property.body.1.is_empty()
        && property.start.1.is_empty() 
        {
            if property.position.1 == 0 
            {
                reterr!(self::Error::InvalidPropertyValue,
                    value.to_string(), 
                    property.body.0,
                    format!(
                        "You buffon, your \"full\" & \"start\" are both \
                        empty yet your specific_pos is 0. Set a specific
                        position or this argument will match everything!"
                    )
                );
            }
        }
        else 
        {
            
            if property.body.1
                .chars()
                .find(|x| x.is_whitespace())
                .is_some()
            {
                reterr!(self::Error::InvalidPropertyValue,
                    value.to_string(), 
                    property.body.0,
                    format!(
                        "You buffon, you whitespaces in this thing, what \
                        is wrong with you?!",
                    )
                );
            }
        
            if !property.body.1
                .starts_with(&property.start.1)
            || property.start.1.len() >= property.body.1.len()
            {
                reterr!(self::Error::InvalidPropertyValue,
                    value.to_string(), 
                    property.body.0,
                    format!(
                        "You buffon, your \"argument\" only contains the \
                        the start portion \"{}\"!",
                        &property.start.1
                    )
                );
            }

            let ch = property.body.1
                .split_at(property.start.1.len())
                .1
                .chars();

            let invalid_ch =
                ch.clone()
                .into_iter()
                .find_position(|x| {
                    if !x.is_alphanumeric()
                    && !property.body_separator.1
                        .is_some_and(|y| x == &y)
                    && !property.value_separator
                        .is_some_and(|y| x == &y)
                    {
                        true
                    } 
                    else 
                    {
                        false
                    }
                });

            if invalid_ch.is_some()
            {
                reterr!(self::Error::InvalidPropertyValue,
                    value.to_string(), 
                    property.body.0,
                    format!(
                        "You buffon, the main body of the argument can \
                        only contain alphanumeric values, the \
                        name separator symbol or the value separator symbol!"
                    )
                );
            }

            let val_sep_count = ch
                .clone()
                .into_iter()
                .filter(|x| x == &VALUE_SEPARATOR_SYMBOL)
                .collect::<Vec<char>>()
                .len();
            if val_sep_count > 1
            {
                reterr!(self::Error::InvalidPropertyValue,
                    value.to_string(), 
                    property.body.0,
                    format!(
                        "You buffon, the Property should only contain \
                        one VALUE_SEPARATOR_SYMBOL => '{}'.",
                        VALUE_SEPARATOR_SYMBOL
                    )
                );
            }
            else if val_sep_count == 1 
            {
                if ch.last().is_some_and(|x| x != VALUE_SEPARATOR_SYMBOL)
                {
                    reterr!(self::Error::InvalidPropertyValue,
                        value.to_string(), 
                        property.body.0,
                        format!(
                            "You buffon, the Property should only contain \
                            one VALUE_SEPARATOR_SYMBOL => '{}' at the end \
                            of the body!",
                            VALUE_SEPARATOR_SYMBOL
                        )
                    );
                }
            }
        }

        return Ok(property);

    }


    #[allow(unused)]
    /// Convert a generic type that implements [`EnumProperty`](strum::EnumProperty) 
    /// & [`Display`](std::fmt::Display) into a vector of keys.
    /// 
    /// # Example
    /// If the Argument does not contain `values` as a property we return a 
    /// vector with a single element.
    /// ```
    /// let keys = Property::generic_to_keys(
    ///     &Override::CompilerName(String::new())
    /// );
    /// assert_eq!(
    ///     keys, 
    ///     Ok(vec![String::from("--compiler.name=")])
    /// );
    /// ```
    /// Otherwise we multiple keys where the `body` and each element from `values` 
    /// are concatenated.
    /// ```
    /// let keys = Property::generic_to_keys(
    ///     &Argument::MessageAmount(String::new())
    /// );
    /// assert_eq!(
    ///     keys, 
    ///     Ok(vec![String::from("-ml"), String::from("-mm"), String::from("-ms")])
    /// );
    /// ```
    pub(super) fn generic_to_keys<T: strum::EnumProperty + std::fmt::Display> (
        value: &T,
    ) -> Result<Vec<String>, self::Error>
    {

        let number = 
        if value
            .get_str(Property::default().position.0)
            .is_some_and(|x| x.parse::<i32>().is_ok() )
        {
            value
                .get_str(Property::default().position.0)
                .unwrap()
                .parse::<i32>()
                .unwrap()
        }
        else {0};

        #[cfg(debug_assertions)]
        let body =
        if value
            .get_str(Property::default().body.0)
            .is_some() 
        {
            value.get_str(Property::default().body.0).unwrap()
        }
        else if number > 0
        {
            ""
        }
        else 
        {
            reterr!(self::Error::InvalidPropertyValue,
                value.to_string(),
                Property::default().body.0,
                format!(
                    "You buffon, you didn't make a \
                    value for the Property!"
                )
            );  
        };
        #[cfg(not(debug_assertions))]
        let body = value.get_str(Property::default().body.0).unwrap();

        let values: Vec<&str> =
        if value
            .get_str(Property::default().values.0)
            .is_some()
        {
            value
                .get_str(Property::default().values.0)
                .unwrap()
                .split(',')
                .filter(|x| !x.is_empty())
                .map(|x| x.trim())
                .collect()
        }
        else {vec![]};

        if number != 0 {Ok(vec![number.to_string()])} 
        else if !values.is_empty()
        {
            Ok(values
                .iter()
                .map(|x| String::from(body.to_owned()+x))
                .collect()
            )
        }
        else {Ok(vec![body.to_owned()])}

    }

    /// Convert property, to a vector of keys.
    /// 
    /// # Example
    /// If the Argument does not contain `values` as a property we return a 
    /// vector with a single element.
    /// ```
    /// let property = Property::generic_try_from(
    ///     &Override::CompilerName(String::new())
    /// );
    /// assert_eq!(
    ///     property.to_keys(), 
    ///     Ok(vec![String::from("--compiler.name=")])
    /// );
    /// ```
    /// Otherwise we multiple keys where the `body` and each element from `values` 
    /// are concatenated.
    /// ```
    /// let property = Property::generic_try_from(
    ///     &Argument::MessageAmount(String::new())
    /// );
    /// assert_eq!(
    ///     property.to_keys(), 
    ///     Ok(vec![String::from("-ml"), String::from("-mm"), String::from("-ms")])
    /// );
    /// ```
    pub(super) fn to_keys (
        &self
    ) -> Result<Vec<String>, self::Error>
    {

        if self.position.1 > 0 {Ok(vec![self.position.1.to_string()])} 
        else if !self.values.1.is_empty()
        {
            Ok(self.values.1
                .iter()
                .map(|x| String::from(self.body.1.clone()+x))
                .collect()
            )
        }
        else {Ok(vec![self.body.1.clone()])}

    }

}

lazy_static!(
    pub(super) static ref PROPERTIES: Mutex<HashMap<String, (Argument, Property)>> 
    = Mutex::new(
        HashMap::new()
    );
);

pub fn print_property_pools() {

    PROPERTIES.lock().unwrap().iter().for_each(|x| println!("[{}]: {:#?}", x.0, x.1.1));

}

/// Fills the property pool, used when matching a String to some Argument and
/// its Property.
/// 
/// #Example:
/// ```
/// let key = String::from("--compiler.name=")
/// assert!(PROPERTIES.lock().unwrap().contains_key(&key), false);
/// 
/// self::fill_property_pools();
/// 
/// assert!(
///     PROPERTIES.lock().unwrap().get(&key).is_some_and(
///         |x: &(Argument, Property)| x.1.body.1 == key)
///     true
/// );
/// ```
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

        let keys = 
        match property.to_keys()
        {
            Ok(res) => {res.into_iter()},
            Err(err) => {return Err(err)}
        };
        
        for key in keys {

            if PROPERTIES
                .lock()
                .unwrap()
                .insert(key.to_owned(), (arg.clone(), property.clone()))
                .is_some()
            {

                reterr!(self::Error::InvalidPropertyValue,
                    arg.to_string(),
                    property.body.0,
                    format!("Two Arguments {}/{} generated the same key/s!",
                        PROPERTIES.lock().unwrap().get(&property.body.1).unwrap().0.to_string(),
                        arg.to_string()
                    )
                );

            }

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

        let keys = 
        match property.to_keys()
        {
            Ok(res) => {res.into_iter()},
            Err(err) => {return Err(err)}
        };
        
        for key in keys {
            
            if PROPERTIES
                .lock()
                .unwrap()
                .insert(key.to_owned(), (Argument::Overrides(arg.clone()), property.clone()))
                .is_some()
            {

                reterr!(self::Error::InvalidPropertyValue,
                    arg.to_string(),
                    property.body.0,
                    format!("Two Arguments {}/{} generated the same key/s!",
                        PROPERTIES.lock().unwrap().get(&property.body.1).unwrap().0.to_string(),
                        arg.to_string()
                    )
                );

            }

        }

    } 

    Ok(())

}

impl Argument {

    /// Handle found arguments and their values.
    /// 
    /// Used to convert the values to their proper type,
    /// modify the value before it inputs it into the argument or use other
    /// strings (arguments) from the vector of probable arguments as input.
    /// 
    /// This function is meant to be only used by `Argument::try_from_string()`
    /// but no one can stop from using it somewhere else.
    /// 
    /// # Example
    /// ```
    /// let arg   = Argument::MessageAmount(String::new());
    /// let prop  = Property::generic_try_from(&arg);
    /// let value = "s"
    /// let full  = prop.body.1.as_str().clone() + value; // -ms
    /// let complete = Argument::from_parts(
    ///     &arg, 
    ///     &prop,
    ///     prop.body.1.as_str(), // -m
    ///     value,                // s
    ///     full,                 // -ms
    ///     None, // Used when reading from a vector. For arguments that take in other arguments as input.
    /// );
    /// assert_eq!(Argument::MessageAmount(String::from("s")), complete);
    /// ```
    fn from_parts(
        arg:      &Self,
        property: &Property,
        name:     &str,
        value:    &str,
        full:     &str,
        iter:     Option<&mut Peekable<std::slice::Iter<String>>>
    ) -> Self {

        match arg
        {
            Self::Files(_) => {
                if iter.is_some() 
                {
                    let uniter = iter.unwrap();
                    let mut arr: Vec<String> = vec![];
                    while let Some(item) = uniter.next() {
                        arr.push(item.clone());
                    }
                    Self::Files(arr)
                }
                else 
                {
                    Self::Files(vec![])
                }
            }
            Self::Overrides(over) => { match over {
                Override::CompilerFlags(_) => {
                    Self::Overrides(Override::CompilerFlags(String::from(value)))
                }
                Override::CompilerLibraries(_) => {
                    Self::Overrides(Override::CompilerLibraries(String::from(value)))
                }
                Override::CompilerName(_) => {
                    Self::Overrides(Override::CompilerName(String::from(value)))
                }
                Override::Unknown(_) => {
                    Self::Overrides(Override::Unknown((
                        String::from(name), 
                        String::from(value),
                        String::from(full)
                    )))
                }
            }}
            Self::MessageAmount(_) => {
                match property.values.1.iter().find(|x| x.as_str() == value)
                {
                    Some(res) => {
                        GLOBALS.write().unwrap().set_message_amount(
                            globals::Degree::from(&value.chars().next().unwrap()),
                            globals::AccessLevel::from_filename(filename!())
                        );
                        Argument::MessageAmount(res.to_owned())
                    }
                    None => {Argument::Unknown((
                        String::from(name), 
                        String::from(value),
                        String::from(full)
                    ))}
                }
            }
            Self::Recipe(_) => {
                Argument::Recipe(String::from(full))
            }
            Self::PrintInstructions => {
                Argument::PrintInstructions
            }
            Self::Warnings => {
                GLOBALS.write().unwrap().set_warn(
                    true, 
                    globals::AccessLevel::from_filename(filename!())
                );
                // eprintln!("{:#?}", globals::AccessLevel::from_filename(filename!()));
                Argument::Warnings
            }
            Self::Unknown(_) => {
                Argument::Unknown((
                    String::from(name), 
                    String::from(value),
                    String::from(full)
                ))
            }
        }

    }
    
    /// Find a similar argument from PROPERTIES with 
    /// [normalized Damerau Levenshtein](normalized_damerau_levenshtein) 
    /// string similarity function.
    fn find_similar(
        name: &String,
        pos: &String,
        it: std::collections::hash_map::Iter<'_, String, (Argument, Property)>
    ) -> Error {

        const SIMILAR_MINIMUM: f64 = 0.90000;

        let mut positional: (&String, &Argument) 
            = (&String::new(), &Argument::default());

        let mut similar:    (&String, &Argument, f64) 
            = (&String::new(), &Argument::default(), 0.00000);

        for items in it {
            if items.0.parse::<u32>().is_ok() {
                if items.0 == pos {positional = (items.0, &items.1.0)}
                continue;   
            }
            let sim = normalized_damerau_levenshtein(
                items.0.as_str(),
                name.as_str()
            ).abs();
            if sim > similar.2 {
                similar.2 = sim;
                similar.1 = &items.1.0;
                similar.0 = items.0;
            }                    
        }
        // Eww, but it's cool?
        if similar.2 > SIMILAR_MINIMUM {
            return Error::UnknownArgument(
                errpos!(),
                name.to_owned(),
                format!("...perhaps you meant to type {}?", 
                    fmterr_val!(similar.0))
            )
        }
        // Wont ever be matched cuz if it was a positional
        // argument then it wont end up here, fuk
        else if !positional.0.is_empty() {
            return Error::UnknownArgument(
                errpos!(),
                name.to_owned(),
                format!("... perhaps you typed your positional \
                    argument {} wrong?", 
                    fmterr_val!(positional.1.to_string()))
            )
        }
        else {
            if !similar.0.is_empty() {
                return Error::UnknownArgument(
                    errpos!(),
                    name.to_owned(),
                    format!("...perhaps you meant to type {}?", 
                        fmterr_val!(similar.0))
                )
            }
            else {
                return Error::UnknownArgument(
                    errpos!(),
                    name.to_owned(),
                    String::new()
                );
            }
        }
    }

    /// Try to convert a string into a Argument.
    /// # Example
    /// ```
    /// assert_eq!(
    ///     Argument::try_from_string(0, &String::from("--compiler.name=gcc"), None), 
    ///     Argument::Overrides(Override::CompilerName(String::from("gcc")))
    /// );
    /// 
    /// assert_eq!(
    ///     Argument::try_from_string(0, &String::from("-f"), None), 
    ///     Argument::Files(vec![])
    /// );
    /// 
    /// let v: Vec<String> = "-f foo.c bar.c".split_whitespace().map(String::from).collect();
    /// let mut it = v.iter().peekable().enumerate();
    /// 
    /// assert_eq!(
    ///     Argument::try_from_string(0, it.next().1, Some(&mut it)), 
    ///     Argument::Files(vec!["foo.c", "bar.c"])
    /// );
    /// 
    /// assert_eq!(
    ///     Argument::try_from_string(1, &String::from("all"), None),
    ///     Argument::Recipe(String::from("all"))
    /// )
    /// ```
    fn try_from_string(
        // p:  usize,   // Position inside the argument vector
        s:  &String, // String slice
        it: Option<&mut Peekable<std::slice::Iter<String>>>,
    ) -> Result<Self, self::Error> {

        static mut UNKNOWNS: Mutex<usize> = Mutex::new(1); // Drop in for detecting 
        // arguments based on purely position, if a argument is unknown
        // then its probably a positional argument.
        // This 
        // let pos = p.to_string();

        if s.is_empty() {
            reterr!(self::Error::UnknownArgument, String::from(s), String::new());
        }
        
        if PROPERTIES
            .lock()
            .unwrap()
            .is_empty() 
        {
            match fill_property_pools() {
                Err(err) => {return Err(err)},
                _ => {}
            }
        }

        let properties_unlocked = PROPERTIES
            .lock()
            .unwrap();

        let clean = s.trim_start();

        let name = 
        match clean
            .split_once('=')
        {
            Some(res) => {res.0.to_owned()+"="},
            None => {clean.to_owned()}
        };

        let bucket = 
        match properties_unlocked
            .get(&name)
        {
            Some(res) => {res.clone()}
            None => {   
                // unsafe {       
                let mut cur_unknowns: usize = unsafe{ UNKNOWNS.lock().unwrap().deref().to_owned() };   
                match properties_unlocked
                    .get(&cur_unknowns.to_string())
                {
                    Some(res) => {
                        cur_unknowns += 1; 
                        unsafe{UNKNOWNS = Mutex::new(cur_unknowns)};
                        res.clone()}
                    None => {
                        // This is the part find the similar argument.
                        return Err(
                            Argument::find_similar(
                                &name, 
                                &cur_unknowns.to_string(), 
                                properties_unlocked.iter()))}                
                }
                // }
            }  
        };

        let arg_val =
        if bucket
            .1
            .body_separator
            .1
            .is_some()
        {
            match clean.split_once('=')
            {
                Some(val) => {val.1}
                None => {unreachable!()}
            }
        }
        else if !bucket
            .1
            .values
            .1
            .is_empty()
        {
            clean.split_at(bucket.1.body.1.len()).1
        }
        else
        {
            ""
        };

        Ok(Argument::from_parts(
            &bucket.0, 
            &bucket.1,
            name.as_str(), 
            arg_val, 
            s.clone().as_str(),
            it))

    }

    /// Generate a sequence of Arguments from a sequence of Strings.
    /// # Example
    /// ```
    /// let cmd_args = vec![
    ///         String::from("all"),
    ///         String::from("-mm"),
    ///         String::from("--compiler.name=clang"),
    ///         String::from("-f"),
    ///         String::from("foo.c"),
    ///         String::from("bar.c"),
    /// ];
    /// 
    /// let args = match Argument::try_from_vec(&cmd_args)
    /// {
    ///     Ok(res) => {res}
    ///     Err(err) => {eprintln!("{}", err); exit(1)}
    /// }
    /// 
    /// assert_eq!(
    ///     res,
    ///     vec![
    ///         Argument::Recipe(String::from("all")),
    ///         Argument::MessageAmount(String::from("m")),
    ///         Argument::Overrides(Override::CompilerName(String::from("clang"))),
    ///         Argument::Files(vec![String::from("foo.c"), String::from("bar.c")]),
    ///     ]
    /// )
    /// ```
    pub fn try_from_vec(v: &Vec<String>) -> Result<Vec<Self>, self::Error> {
        
        let mut iter: Peekable<std::slice::Iter<'_, String>> = v.into_iter().peekable();
        let mut args: Vec<Self> = vec![];
        iter.next();
        
        while let Some(chunk) = iter.next(){

            match Argument::try_from_string( chunk, Some(&mut iter))
            {
                Ok(res) => {args.push(res)}
                Err(err) => {return Err(err)}
            }

        }

        Ok(args)

    }

    // pub fn try_from_config(v: &Config) {

    // }

}
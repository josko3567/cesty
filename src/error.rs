//! Custom error and warning type used by cesty named [Alert].

use colored::{
    ColoredString, 
    Colorize
};

use lazy_static::lazy_static;

use std::{
    env, fmt::Debug
};


#[derive(Debug, Default, Clone)]
pub struct AlertCodeFix {

    pub relative_line:   usize,
    pub column:          usize,
    pub comment:        String,

}

#[derive(Debug, Default, Clone)]
pub struct AlertCode {

    pub line: usize,
    pub code: String,
    pub file: String,

    pub fix: Vec<AlertCodeFix>

}

#[derive(Debug, Clone)]
pub enum AlertExample {

    Code(AlertCode)

}


#[derive(Debug, Default, Clone)]
pub struct AlertDebug {
    
    pub line:     usize,
    pub column:   usize,
    
    pub file:     &'static str,
    pub function: &'static str,

}

#[derive(Debug, Default, Clone)]
pub struct AlertInfo {
    
    pub description: String,
    pub note:        Vec<String>,
    pub example:     Option<AlertExample>,
    pub debug:       Vec<AlertDebug>,

    
}

#[derive(Debug, Clone)]
pub enum Alert {
    
    Warning(AlertInfo),
    Error(AlertInfo)
    
}

impl Default for AlertExample {

    fn default() -> Self {
        AlertExample::Code(AlertCode::default())
    }

}

impl Default for Alert {

    fn default() -> Self {
        Alert::Error(AlertInfo::default())
    }

}

lazy_static! {

    /// Terminal tab size, currently unused.
    static ref TERMINAL_TABSIZE: usize = match env::var("TABSIZE") {
        Ok(s) => s.parse::<usize>().unwrap_or(8),
        Err(_) => 8
    };

}

/// Custom spacing for the alert examples.
fn get_spacing(line: usize) -> usize {

    const MINIMUM_SIZE: usize = 3;
    let   number_size:  usize = format!("{}", line).len();

    if MINIMUM_SIZE / 2 < number_size {
        MINIMUM_SIZE - MINIMUM_SIZE/2 + number_size
    } else {
        MINIMUM_SIZE
    }

}

/// Create a string of **n** spaces.
/// 
/// # Example
/// ```
/// assert!("    ".to_owned(), spacing_to_string(4));
/// ```
fn spacing_to_string(n: usize) -> String {

    let mut s: String = String::new();
    (0..n).for_each(|_| s.push(' '));
    s

}

/// Create a string of spaces that follows a string even if 
/// the string contains a tab.
/// 
/// # Example
/// ```
/// let my_str1: String = "hello world!".to_owned();
/// let my_str2: String = "hel".to_owned();
/// let my_str3: String = "he\tllo world!".to_owned();
/// assert!("    ".to_owned(),  spacing_from_string(4, &my_str1));
/// assert!("   ".to_owned(),   spacing_from_string(4, &my_str2));
/// assert!("  \t ".to_owned(), spacing_from_string(4, &my_str3));
/// ```
fn spacing_from_string(spacing: usize, s: &String) -> String {

    let mut spaces = String::new();
    let mut iter = s.chars().into_iter();
    for _ in 1..spacing {
        let ch = match iter.next() {
            Some(ch) => ch,
            None           => break
        };
        if ch == '\t' {
            spaces.push('\t')
        } else {
            spaces.push(' ')
        }
    }
    spaces

}

impl AlertCode {

    /// Custom formatter that accepts a `&Alert` to indicate how
    /// to color some parts of the message.
    fn custom_fmt(&self, f: &mut std::fmt::Formatter<'_>, alert: &Alert, spacing: usize) -> std::fmt::Result {

        let code_lines: Vec<String> = self.code
            .lines()
            .map(String::from)
            .collect();
        
        // "--> `filename.yaml.cesty`:45"
        writeln!(f, "{}{}", 
            format!("{}--> ", 
                spacing_to_string(spacing-1),
            ).blue().bold(),

            format!("`{}`:{}", 
                self.file,
                self.line
            ).normal().italic()
        )?;
        
        // First "    | "
        writeln!(f, "{}",
            format!("{}|", 
                spacing_to_string(spacing), 
            ).bold().blue()
        )?;

        let mut fix_iter = self.fix.iter().peekable();
        while let Some(fix) = fix_iter.next() {

            let line: usize = self.line;
            // println!("line");

            // Get code snippet from relative line.
            let snippet: ColoredString = if code_lines.get(fix.relative_line).is_some() 
            {
                code_lines.get(fix.relative_line).unwrap().to_owned().normal()
            } 
            else 
            {
                "{out of bounds}".to_owned().italic()
            };

            // Snippet aka. "    | let lorem: ipsum = 5;"
            writeln!(f, "{}{}",
                format!("{}{} |", 
                    spacing_to_string(spacing-1-format!("{line}").len()), 
                    line
                ).bold().blue(),

                format!("    {}", snippet).normal()
            )?;

            // Fix aka. "    |  ^ Lorem ipsum."
            match alert {
                Alert::Warning(_) => {
                    writeln!(f, "{}{}",
                        format!("{}|", 
                            spacing_to_string(spacing), 
                        ).bold().blue(),

                        format!("    {}^ {}", 
                            spacing_from_string(fix.column, &snippet.to_string()),
                            fix.comment
                        ).yellow().bold()
                    )?;
                },
                Alert::Error(_) => {
                    writeln!(f, "{}{}",
                        format!("{}|", 
                            spacing_to_string(spacing), 
                        ).bold().blue(),

                        format!("    {}^ {}", 
                            spacing_from_string(fix.column, &snippet.to_string()),
                            fix.comment
                        ).red().bold()
                    )?;
                }
            }

            // ... placed for multi connected issues.
            if fix_iter.peek().is_some() {
                writeln!(f, "{}", 
                    format!("...", ).bold().blue()
                )?;
            }

        }

        Ok(())

    }

}

impl std::fmt::Display for Alert {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let alertinfo = match self {

            Alert::Error(info)   => info,
            Alert::Warning(info) => info

        };

        #[cfg(debug_assertions)] {
            writeln!(f, "{}", format!("Call stack trace:").dimmed().italic())?;
            for debug in alertinfo.debug.clone() {
                writeln!(f, "{}", 
                    format!("  {:>16} => {}[{}:{}].", 
                        debug.file.bold(),
                        debug.function.bold(), 
                        format!("{}", debug.line).bold(),
                        format!("{}", debug.column).bold(),
                    ).dimmed().italic()
                )?
            } 
        }

        match self {

            Alert::Warning(_) => write!(f, "{}{} ", "warning".bold().yellow(), ":".bold())?,
            Alert::Error(_)   => write!(f, "{}{} ", "error".bold().red(),      ":".bold())?

        }

        let spacing: usize = {

            let mut size: usize = 1;

            if alertinfo.example.is_some() {

                match alertinfo.example.as_ref().unwrap() {
                    AlertExample::Code(code) => {
                        let mut largest: usize = 0;
                        for i in code.fix.iter() {
                            if i.relative_line > largest {
                                largest = i.relative_line
                            }
                        }

                        size = code.line + largest;
                    }

                }

            }

            get_spacing(size)
        };

        write!(f, "{}\n", alertinfo.description.bold())?;
        if alertinfo.example.is_some() {
            match alertinfo.example.as_ref().unwrap() {
                AlertExample::Code(code) => code.custom_fmt(f, &self, spacing)?,
            }

            // First "    | "
            writeln!(f, "{}",
                format!("{}|", 
                    spacing_to_string(spacing), 
                ).bold().blue()
            )?;
        
        } else {

            // First "    | "
            writeln!(f, "{}",
                format!("{}?", 
                    spacing_to_string(spacing), 
                ).bold().blue()
            )?;

        }
    
        for note in alertinfo.note.iter() {

            // First "    | "
            writeln!(f, "{} {} {}",
                format!("{}=", 
                    spacing_to_string(spacing), 
                ).bold().blue(),
                format!("note:").bold(),
                note
            )?;

        }

        

        return Ok(());

    }
}

impl std::error::Error for Alert {}

impl Alert {

    pub(crate) fn push_debug(mut self, debug: AlertDebug) -> Self {
        match self {
            Alert::Error(ref mut info)   => info.debug.push(debug),
            Alert::Warning(ref mut info) => info.debug.push(debug)
        }
        self
    }

    pub(crate) fn from_serde_yaml(
        err:         serde_yaml::Error,
        description: String,
        filename:    std::path::PathBuf,
        debug:       Vec<AlertDebug>
    ) -> Self {

        match err.location() {

            Some(location) => {

                let file_contents: String = match std::fs::read_to_string(&filename) {
                    Ok(str) => str,
                    Err(_) => { 
                        return Alert::Error( AlertInfo {
                            description: description,
                            debug: debug,
                            example: None,
                            note: vec![
                                "failed to parse the file for a more in depth error message".to_owned(),
                                "serde_yaml::Error contains the following message:".to_owned(),
                                err.to_string()
                            ]
                        })
                    }
                };

                return Alert::Error( AlertInfo {
                    description: description,
                    debug: debug,
                    example: Some(AlertExample::Code(AlertCode {
                        line: location.line(),
                        file: filename.to_string_lossy().to_string(),
                        code: file_contents,
                        fix:  vec![AlertCodeFix {
                            relative_line: location.line()-1,
                            column: location.column(),
                            comment: err.to_string()
                        }]
                    })),
                    note: vec![]
                })
            }
            None => {
                return Alert::Error( AlertInfo {
                    description: description,
                    debug: debug,
                    example: None,
                    note: vec![
                        "serde_yaml::Error contains the following message:".to_owned(),
                        err.to_string()
                    ]
                })
            }

        }

    }

    pub(crate) fn from_toml(
        err:         toml::de::Error,
        description: String,
        filename:    std::path::PathBuf,
        debug:       Vec<AlertDebug>
    ) -> Self {

        match err.span() {

            Some(location) => {

                let file_contents: String = match std::fs::read_to_string(&filename) {
                    Ok(str) => str,
                    Err(_) => { 
                        return Alert::Error( AlertInfo {
                            description: description,
                            debug: debug,
                            example: None,
                            note: vec![
                                "failed to parse the file for a more in depth error message".to_owned(),
                                "serde_yaml::Error contains the following message:".to_owned(),
                                err.to_string()
                            ]
                        })
                    }
                };

                let start_lines = (&file_contents[0..location.start])
                    .lines()
                    .collect::<Vec<&str>>();

                let line = start_lines.len();

                let column = if start_lines.last().is_some() {
                    start_lines.last().unwrap().len()
                } else {
                    0
                };

                let comment = err
                    .message()
                    .to_owned()
                    .replace("\n", " ")
                    .trim()
                    .to_owned() 
                + format!(" on line {}, column {}.", line, column).as_str();

                return Alert::Error( AlertInfo {
                    description: description,
                    debug: debug,
                    example: Some(AlertExample::Code(AlertCode {
                        line: line,
                        file: filename.to_string_lossy().to_string(),
                        code: file_contents,
                        fix:  vec![AlertCodeFix {
                            relative_line: line-1,
                            column: column,
                            comment: comment
                        }]
                    })),
                    note: vec![]
                })
            }
            None => {
                return Alert::Error( AlertInfo {
                    description: description,
                    debug: debug,
                    example: None,
                    note: vec![
                        "serde_yaml::Error contains the following message:".to_owned(),
                        err.message().replace("\n", " ").trim().to_owned()
                    ]
                })
            }

        }

    }

}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }}
}

pub(crate) use function;


macro_rules! debuginfo {
    () => {
        vec![crate::error::AlertDebug {
            line: line!() as usize,
            column: column!() as usize,
            file: std::path::Path::new(file!()).file_name().and_then(|s| s.to_str()).unwrap(),
            function: crate::error::function!()
        }]
    };
}
pub(crate) use debuginfo;


macro_rules! debugpush {
    ($err:expr) => {
        $err.push_debug(crate::error::AlertDebug {
            line: line!() as usize,
            column: column!() as usize,
            file: std::path::Path::new(file!()).file_name().and_then(|s| s.to_str()).unwrap(),
            function: crate::error::function!()
        })
    };
}
pub(crate) use debugpush;


macro_rules! debugappend {
    ($err:expr) => {
        $err.into_iter().map(|x| x.push_debug(crate::error::AlertDebug {
            line: line!() as usize,
            column: column!() as usize,
            file: std::path::Path::new(file!()).file_name().and_then(|s| s.to_str()).unwrap(),
            function: crate::error::function!()
        })).collect::<Vec<Alert>>()
    };
}
pub(crate) use debugappend;


macro_rules! error {
    {$( $it:ident : $value:expr) ,*} => {
        Err(crate::error::Alert::Error( AlertInfo {$( $it: $value ),*} ))
    };
}
pub(crate) use error;


macro_rules! warning {
    {$( $it:ident : $value:expr) ,*} => {
        crate::error::Alert::Warning( AlertInfo {$( $it: $value ),+} )
    };
}
pub(crate) use warning;


macro_rules! function_message {
    ($func:expr, $msg:expr) => {
        vec![
            format!("{} returned the following message:", $func),
            format!("{}", $msg)
        ]
    };
}
pub(crate) use function_message;

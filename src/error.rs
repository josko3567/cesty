use std::str::FromStr;
use strum_macros::*;

#[repr(u8)]
#[allow(dead_code)]
#[derive(EnumString)]
pub enum ErrorGroup {
    #[strum(serialize="unknown file")]
    Unknown,

    #[strum(serialize="argument.rs")]
    Argument,

    #[strum(serialize="config.rs")]
    Config,

    #[strum(serialize="lister.rs")]
    Lister,

    #[strum(serialize="extract.rs")]
    Extract,

    #[strum(serialize="clang.rs")]
    Clang,

    #[strum(serialize="environment.rs")]
    Environment,

    #[strum(serialize="translate.rs")]
    Translate

}

impl ErrorGroup {
    
    pub fn from(value: &str) -> Self {
    
        match ErrorGroup::from_str(value) {
            Ok(val) => {val}
            _ => {ErrorGroup::Unknown}
        }
        
    }

}

pub(super) fn type_name<T>(_: &T) -> String { 
    format!("{}", std::any::type_name::<T>())
}

#[derive(Debug, Default, Clone)]
pub struct ErrorPosition {

    pub file:     String,
    pub function: String,
    pub line:     u32,
    pub column:   u32

}

// const color_var: CustomColor = CustomColor {r: 252, b: 162, g:3};

/// From lister.rs // Dimmed
/// Warning! // Yellow, Bold.
/// \t ... /// Yellow, yellow bold for $($x:expr),*
#[allow(unused_macros)]
macro_rules! fmtwarn {
    
    ($short:literal, $full:literal $(, $x:expr)*) => {


        format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                Path::new(file!()).file_name() 
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .bold(),

                function!().bold(),
                format!("{}",line!()).bold(), 
                format!("{}",column!()).bold()
            ).dimmed(),

            "Warning: "
                .bold()
                .yellow(),

            format!($short)
                .replace("\n", "")
                .bold(),
            formatdoc!(
                    $full 
                    $(, $x)*
                )
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| "  ".to_string() + x + "\n".to_owned().as_str())
                .collect::<Vec<String>>()
                .concat()

        ).normal()



    };
}

macro_rules! fmtwarnp {
    
    ($pos:expr, $short:literal, $full:literal $(, $x:expr)*) => {

        format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                $pos.file.bold(),
                $pos.function.bold(),
                format!("{}",$pos.line).bold(), 
                format!("{}",$pos.column).bold()
            ).dimmed(),

            "Warning: "
                .bold()
                .yellow(),

            format!($short)
                .replace("\n", "")
                .bold(),

            formatdoc!(
                    $full 
                    $(, $x)*
                )
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| "  ".to_string() + x + "\n".to_owned().as_str())
                .collect::<Vec<String>>()
                .concat()

        ).normal()


    };
}

macro_rules! warn {
    ($short:literal, $full:literal $(, $x:expr)*) => {


        eprintln!("{}", format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                Path::new(file!()).file_name() 
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .bold(),

                function!().bold(),
                format!("{}",line!()).bold(), 
                format!("{}",column!()).bold()
            ).dimmed(),

            "Warning: "
                .bold()
                .yellow(),

            format!($short)
                .replace("\n", "")
                .bold(),
            formatdoc!(
                    $full 
                    $(, $x)*
                )
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| "  ".to_string() + x + "\n".to_owned().as_str())
                .collect::<Vec<String>>()
                .concat()

        ).normal())

    };

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

macro_rules! errpos {

    () => {
        
        ErrorPosition {
            file: Path::new(file!()).file_name() 
                .and_then(|s| s.to_str())
                .unwrap()
                .to_string(),
            function: function!().to_string(),
            line: line!(),
            column: column!()
        }
    }

}

macro_rules! fmterr_val {

    ($x:expr) => {

        format!(
            "{} {}{}{}{}{}", 
            // "val.".dimmed().strikethrough(),
            $x.to_string().blue().bold(), 
            "invar. ".dimmed().strikethrough(),
            stringify!($x).bold().truecolor(5, 158, 66).dimmed().strikethrough(),
            " of type ".dimmed().strikethrough(),
            crate::error::type_name(&$x).bold().truecolor(5, 158, 66).italic().dimmed().strikethrough(),
            "...".dimmed().strikethrough()
        )

    };

}
macro_rules! fmterr_name {

    ($x:expr) => {

        format!(
            "{}{}{}{}{}", 
            // "val.".dimmed().strikethrough(),
            // $x.to_string().blue().bold(), 
            "invar. "
                .dimmed()
                .strikethrough(),
            stringify!($x)
                .bold()
                .truecolor(5, 158, 66)
                .dimmed()
                .strikethrough(),
            " of type "
                .dimmed()
                .strikethrough(),
            crate::error::type_name(&($x))
                .bold()
                .truecolor(5, 158, 66)
                .italic()
                .dimmed()
                .strikethrough(),
            "..."
                .dimmed()
                .strikethrough()
        )

    };

}

macro_rules! fmterr_func {

    ($x:expr) => {

        format!("{}", stringify!($x).bold().truecolor(235, 149, 52))

    }

}

#[allow(unused_macros)]
macro_rules! fmterr {

    ($short:literal, $full:literal $(, $x:expr)*) => {

        // From ... at ...
        //       ˇ    $"Error: "
        //       ˇ    ˇ $short
        //       ˇ    ˇ ˇ    $long
        //       ˇ    ˇ ˇ    ˇ
        format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                Path::new(file!()).file_name() 
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .bold(),

                function!().bold(),
                format!("{}",line!()).bold(), 
                format!("{}",column!()).bold()
            ).dimmed(),

            "Error: "
                .bold()
                .red(),

            format!($short)
                .replace("\n", "")
                .bold(),
            formatdoc!(
                    $full 
                    $(, $x)*
                )
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| "  ".to_string() + x + "\n".to_owned().as_str())
                .collect::<Vec<String>>()
                .concat()

        ).normal()


    };
}

macro_rules! fmtperr {

    ($pos:expr, $short:literal, $full:literal $(, $x:expr)*) => {

        // From ... at ...
        //       ˇ    $"Error: "
        //       ˇ    ˇ $short
        //       ˇ    ˇ ˇ    $long
        //       ˇ    ˇ ˇ    ˇ
        format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                $pos.file.bold(),
                $pos.function.bold(),
                format!("{}",$pos.line).bold(), 
                format!("{}",$pos.column).bold()
            ).dimmed(),

            "Error: "
                .bold()
                .red(),

            format!($short)
                .replace("\n", "")
                .bold(),

            formatdoc!(
                    $full 
                    $(, $x)*
                )
                .split('\n')
                .filter(|x| !x.is_empty())
                .map(|x| "  ".to_string() + x + "\n".to_owned().as_str())
                .collect::<Vec<String>>()
                .concat()

        ).normal()
    }

}

macro_rules! reterr {
    ($t:expr $(,$vals:expr)*) => {

        return Err($t(errpos!() $(, $vals)*))

    };
}
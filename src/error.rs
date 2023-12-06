/// The source from where the error was thrown.
/// # Example
/// Use the [`errpos!`] macro to get the current `ErrorPosition`.
/// ```
/// let pos: ErrorPosition = errpos!();
/// ```
#[derive(Debug, Default, Clone)]
pub struct ErrorPosition {
    
    pub file:     String,
    pub function: String,
    pub line:     u32,
    pub column:   u32
    
}

/// Return a [`String`] name for the argument type.
pub(super) fn type_name<T>(_: &T) -> String { 
    format!("{}", std::any::type_name::<T>())
}

/// Return a [`String`] name of the function.
macro_rules! attain_function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap()
    }}
}

/// Formats a warning and the position listed as the source
/// is the line, column & file from where `fmtwarn!` was called.
/// # Argument
/// - The first argument is a short-hand version of the warning.
/// - The second argument is a full description of the warning.
///   It can accept variadic arguments and
///   uses `{}` for displaying them.
/// 
/// # Example
/// ```
/// eprintln!("{}", fmtwarn!(
///     "No arguments found!",
///     "
///         No arguments were passed to cesty.
///           Using default recipe.
///     "
/// ));
/// ```
/// Outputs:
/// ```none
/// (dimmed ⚪) From (dimmed bold ⚪) file.rs...
/// (dimmed ⚪) In (dimmed bold ⚪) foo at (dimmed bold ⚪) 13:37...
/// (🟡)Warning: (bold ⚪) No arguments found!
///   (⚪)No arguments passed to cesty.
///     (⚪)Using default recipe.
/// ```
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

/// Formats a warning and the position listed is given as the first
/// argument.
/// # Argument
/// - The first argument the a [`ErrorPosition`].
/// - The second argument is a short-hand version of the warning.
/// - The third argument is a full description of the warning.
///   It can accept variadic arguments and
///   uses `{}` for displaying them.
/// 
/// # Example
/// ```
/// eprintln!("{}", fmtpwarn!(
///     errpos!(),
///     "No arguments found!",
///     "
///         No arguments were passed to cesty.
///           Using default recipe.
///     "
/// ));
/// ```
/// Outputs:
/// ```none
/// (dimmed ⚪) From (dimmed bold ⚪) file.rs...
/// (dimmed ⚪) In (dimmed bold ⚪) foo at (dimmed bold ⚪) 13:37...
/// (🟡)Warning: (bold ⚪) No arguments found!
///   (⚪)No arguments passed to cesty.
///     (⚪)Using default recipe.
/// ```
macro_rules! fmtpwarn {
    
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

/// Prints a warning and the position listed as the source
/// is the line, column & file from where `warn!` was called.
/// # Argument
/// - The first argument is a short-hand version of the warning.
/// - The second argument is a full description of the warning.
///   It can accept variadic arguments and
///   uses `{}` for displaying them.
/// 
/// # Example
/// ```
/// warn!(
///     "No arguments found!",
///     "
///         No arguments were passed to cesty.
///           Using default recipe.
///     "
/// );
/// ```
/// Outputs:
/// ```none
/// (dimmed ⚪) From (dimmed bold ⚪) file.rs...
/// (dimmed ⚪) In (dimmed bold ⚪) foo at (dimmed bold ⚪) 13:37...
/// (🟡)Warning: (bold ⚪) No arguments found!
///   (⚪)No arguments passed to cesty.
///     (⚪)Using default recipe.
/// ```
macro_rules! warn {
    ($short:literal, $full:literal $(, $x:expr)*) => {


        eprintln!("{}", format!("{}\n{}{}\n{}",

            format!("From {}...\nIn function {} at {}:{}...",
                Path::new(file!()).file_name() 
                    .and_then(|s| s.to_str())
                    .unwrap()
                    .bold(),

                attain_function_name!().bold(),
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

/// Returns a [`ErrorPosition`] for the current line.
macro_rules! errpos {

    () => {
        ErrorPosition {
            file: Path::new(file!()).file_name() 
                .and_then(|s| s.to_str())
                .unwrap()
                .to_string(),
            function: attain_function_name!().to_string(),
            line: line!(),
            column: column!()
        }
    }

}

/// Formats a `variable` such that its value, name and type
/// are contained in the returned [`String`].
/// # Example
/// ```
/// let i: i32 = 50;
/// eprintln!(fmterr_val!(i));
/// ```
/// Outputs:
/// ```none
/// (bold 🔵) value (dimmed strikethrough) invar. (🟢) variable name of type (🟢) type
/// ```
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

/// Formats a `variable` such that its name and type
/// are contained in the returned [`String`].
/// # Example
/// ```
/// let i: i32 = 50;
/// eprintln!(fmterr_val!(i));
/// ```
/// Outputs:
/// ```none
/// (dimmed strikethrough) invar. (🟢) variable name of type (🟢) type
/// ```
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

/// Formats a `function` such that its name
/// is stored in the returned [`String`].
/// # Example
/// ```
/// let i: i32 = 50;
/// eprintln!(fmterr_val!(i));
/// ```
/// Outputs:
/// ```none
/// (🟠) function name
/// ```
macro_rules! fmterr_func {

    ($x:expr) => {

        format!("{}", stringify!($x).bold().truecolor(235, 149, 52))

    }

}

/// Formats a error and the position listed as the source
/// is the line, column & file from where `fmterr!` was called.
/// # Argument
/// - The first argument is a short-hand version of the error.
/// - The second argument is a full description of the error.
///   It can accept variadic arguments and
///   uses `{}` for displaying them.
/// 
/// # Example
/// ```
/// eprintln!("{}", fmterr!(
///     "No arguments found!",
///     "
///         No arguments were passed to cesty.
///     "
/// ));
/// ```
/// Outputs:
/// ```none
/// (dimmed ⚪) From (dimmed bold ⚪) file.rs...
/// (dimmed ⚪) In (dimmed bold ⚪) foo at (dimmed bold ⚪) 13:37...
/// (🔴)Error: (bold ⚪) No arguments found!
///   (⚪)No arguments passed to cesty.
/// ```
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

                attain_function_name!().bold(),
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

/// Formats a error and the position listed is given as the first
/// argument.
/// # Argument
/// - The first argument the a [`ErrorPosition`].
/// - The second argument is a short-hand version of the error.
/// - The third argument is a full description of the error.
///   It can accept variadic arguments and
///   uses `{}` for displaying them.
/// 
/// # Example
/// ```
/// eprintln!("{}", fmtperr!(
///     errpos!(),
///     "No arguments found!",
///     "
///         No arguments were passed to cesty.
///     "
/// ));
/// ```
/// Outputs:
/// ```none
/// (dimmed ⚪) From (dimmed bold ⚪) file.rs...
/// (dimmed ⚪) In (dimmed bold ⚪) foo at (dimmed bold ⚪) 13:37...
/// (🔴)Error: (bold ⚪) No arguments found!
///   (⚪)No arguments passed to cesty.
/// ```
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

/// Custom error return for cesty functions.
macro_rules! reterr {
    ($t:expr $(,$vals:expr)*) => {

        return Err($t(errpos!() $(, $vals)*))

    };
}
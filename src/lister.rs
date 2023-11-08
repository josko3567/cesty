use crate::{
    config::Config, 
    argument::Argument,
    error::{ErrorGroup, ErrorPosition}
};

use std::{
    fmt::Display, 
    ffi::OsString, 
    path::{Path, PathBuf}
};

use globwalk;
use itertools::Itertools;
use path_clean::PathClean;
use indoc::formatdoc;
use colored::Colorize;

// const ERROR_GROUP: u32 = 3;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum Error {
    NoFilesFound(ErrorPosition),
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
            unsafe { *(self as *const Self as *const u8) }
        );
    }
    
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::NoFilesFound(pos) => {
                fmtperr!(pos,
                "No files to parse!",
                "
                    No files to parse from the config and command 
                    line arguments.
                "
                )}
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for Error {}

#[derive(Hash, PartialEq, Clone, Default)]
pub struct ListerFile {

    pub useconf: bool,
    pub path: OsString

}

/// Returns maximum depth globwalker is meant to search.
/// 
/// # Arguments
/// * `fullpath` must be a root path with either a wildcard like *.* or a file 
/// in its last entry.
/// 
/// # Examples
/// ```
/// assert_eq!(get_max_depth(PathBuf::new("/home/foobar/*/*.*")), Some(2));
/// assert_eq!(get_max_depth(PathBuf::new("/home/foobar/*.*")), Some(1));
/// assert_eq!(get_max_depth(PathBuf::new("/home/foobar/rust/cesty/*/*/*.*")), Some(3));
/// ```
fn get_max_depth(fullpath: &PathBuf) -> Option<usize> {

    let mut clone = fullpath.clone();
    let mut pops: usize = 0;

    loop {
        if  clone.is_dir() {break}
        if !clone.pop()    {return None}
        pops += 1;
    }

    return Some(pops);
    
    // let str = cln.to_string_lossy().to_string();
    // let filestr: String = fullpath.to_string_lossy().to_string();
    // println!("{} + {}", str, &filestr[str.len()..filestr.len()]);
    // Some(filestr[str.len()..filestr.len()].matches(&['/', '\\']).count())

}

/// Get apsolute path from either apsolute path or a relative path.
/// 
/// # Example
/// ```
/// // In directory /home/foo/bar
/// assert_eq!(absolute_path(Path::new("/home/foo/bar/par")), PathBuf::new("/home/foo/bar/par"))
/// assert_eq!(absolute_path(Path::new("par")), PathBuf::new("/home/foo/bar/par"))
/// assert_eq!(absolute_path(Path::new("./../bar/.par")), PathBuf::new("/home/foo/bar/./../bar/.par"))
/// ```
#[allow(dead_code)]
fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {

    let path = path.as_ref();
    // Path::new("ea");
    let absolute_path = if path.is_absolute() {

        path.to_path_buf()

    } else {

        std::env::current_dir()?.join(path)

    }
    .clean();

    Ok(absolute_path)

}

// fn get_from_conf()

/// Get a vector of [`ListerFile`] from the [`Config`] and [`Vec<Argument>`](`Argument`).
/// # Example
/// In *fs:*
/// ```
/// .
/// └── home/
///     └── foo/
///         └── bar/
///             ├── cesty.yaml
///             └── src/
///                 ├── fizz.c
///                 ├── buzz.c
///                 ├── foo.c
///                 └── bar.c
/// ```
/// In *cesty.yaml:*
/// ```yaml
/// ...
/// recipe: 
///  - name: src
///    run:
///    - path: src/*.c
///      recurse: false 
/// ...
/// ```
/// *Test:*
/// 
/// ```rust
/// let args: Vec<argument::Argument> = 
/// vec![
///     argument::Argument::Recipe("src"), 
///     argument::Argument::Files(vec!["src/fizz.c", "src/buzz.c"])
/// ];
/// 
/// let mut config = config::Config::new();
/// config.from_file(config::find())?;
/// 
/// let list = lister::get_list(&config, &args)?;
/// 
/// // If the file is in conf & arg, useconf is true.
/// let res = vec![
///     ListerFile {
///         useconf: true, 
///         path: OsString::new("/home/foo/bar/src/fizz.c")}
///     ListerFile {
///         useconf: true, 
///         path: OsString::new("/home/foo/bar/src/buzz.c")}
///     ListerFile {
///         useconf: true, 
///         path: OsString::new("/home/foo/bar/src/foo.c")}
///     ListerFile {
///         useconf: true, 
///         path: OsString::new("/home/foo/bar/src/bar.c")}
/// ];
/// 
/// res.sort_by(|a,b| a.path.cmp(&b.path))
/// list.sort_by(|a,b| a.path.cmp(&b.path))
/// 
/// assert_eq!(res, list);
/// ```
#[allow(non_snake_case)]
pub fn get_list(conf: &Config, args: &Vec<Argument>) -> Result<Vec<ListerFile>, Error> {

    let arg_files = match args
        .iter()
        .find(|x| match x 
        {
            Argument::Files(_files) => {true}
            _ => {false}
        }
    ) 
    {
        Some(arg) => match &arg {
            &Argument::Files(files) => {Some(files)},
            _ => None
        },
        _ => None
    };
    

    let binding_recipe_empty = Argument::Recipe(String::from(""));
    let binding_string_empty = String::from("");

    let arg_recipe_name = match args
        .iter()
        .find(|x| match x {

            Argument::Recipe(_name) => {true}
            _ => {false}

        })
        .unwrap_or(&binding_recipe_empty) 
    {

        Argument::Recipe(name) => {name},
        _ => {
            &binding_string_empty
        }

    };


    
    let arg_recipe = if conf.recipe.is_some() { 
    conf.recipe
        .as_ref()
        .unwrap()
        .iter()
        .find(|x| 
            &x.name == arg_recipe_name
        )
    } else {
        None
    };

    if arg_recipe_name.is_empty() {
        warn!(
            "Did not find a recipe!",
            "
                No recipe was passed through cmd-line arguments.
            "
        );
    } else if arg_recipe.is_none() {
        warn!(
            "Did not find a recipe!",
            "
                No recipe with the name...
                  {}
                ... was found.
            ",
                fmterr_val!(arg_recipe_name)
        );
    }

    let mut files = Vec::<ListerFile>::new();

    if arg_recipe.is_some() {

        let unwrap_recipe = arg_recipe.unwrap();
        
        for run in unwrap_recipe.run.iter() {
            
            let recurse = run.recurse.is_some_and(|x|x);
            let symlink = run.symlinks.is_some_and(|x|x);

            // Fullpath of the recipe.run[?].path variable
            let mut fullpath= if Path::new(&run.path).is_absolute() 
            {
                PathBuf::from(&run.path)
            } 
            else 
            {
                let mut confpath = PathBuf::from(&conf.path);

                // Removes .cesty.{conf,yaml,yml}
                confpath.pop();

                // Pushes local dir/file reference
                confpath.push(&run.path);

                match Path::new(&confpath).is_absolute() {
                    true => {confpath}
                    _ => { 
                        warn!(
                        "Failed to extract absolute path!",
                        "
                            Could not extract a absolute path from...
                                {}
                            ...the path that was extracted is...
                                {}
                        ",
                            fmterr_val!(run.path), 
                            fmterr_val!(confpath.display())
                        );
                        break;
                    }
                }
            };
            
            // Check if path points to file... 
            if fullpath.is_file() {
                files.push(ListerFile{
                    useconf: true,
                    path: fullpath.as_os_str().to_os_string()
                });
                continue;
            }

            // if fullpath is a directory or is a wildcard of directories
            // like /home/.../* we then 
            // append *.* to read actual files from the directory/ies.
            if fullpath.is_dir() 
            || fullpath.file_name().is_some_and(|x| x == "*") {
                fullpath.push("*.*");
            }
        
            let max_depth = if recurse {usize::MAX} else {
                match get_max_depth(&fullpath) {
                    Some(depth) => {depth}
                    _ => {
                        warn!(
                        "Failed to extract max depth!",
                        "
                            Could not extract the maximum depth of file...
                                {}
                            ...with fullpath of...
                                {}
                        ",
                            fmterr_val!(run.path), 
                            fmterr_val!(fullpath.to_string_lossy().to_string())
                        );
                        continue;
                    }
                }

            };

            let min_depth = 0;

            match globwalk::glob_builder(fullpath.to_string_lossy())
                .min_depth(min_depth)
                .max_depth(max_depth)
                .follow_links(symlink)
                .build()
            {
                Ok(res) => {res}
                Err(err) => {
                    warn!(
                    "Failed to extract max depth!",
                    "
                        Failed to open file...
                            {}
                        ...with fullpath of...
                            {}
                        ...because of GlobError...
                            {}
                    ",
                        fmterr_val!(run.path),
                        fmterr_val!(fullpath.display()),
                        err.to_string().bold()
                    );
                    continue;
                }   

            }
            .for_each(|x| { match x {
                Ok(res) => { 
                    if res.path().is_file() {
                        files.push(ListerFile{
                            useconf: true,
                            path: res.path().as_os_str().to_os_string()
                        })
                    }
                },
                Err(err) => {
                    warn!(
                    "Failed to extract max depth!",
                    "
                        Failed to open file...
                            {}
                        ...with fullpath of...
                            {}
                        ...because of Error...
                            {}
                    ",
                        fmterr_val!(run.path),
                        fmterr_val!(fullpath.display()),
                        err.to_string().bold()
                    );
                }
            }});

        }

    }
    
    // Inlist arg files into the ListerFile vector.
    if arg_files.is_some() {

        arg_files.unwrap().iter().for_each(|str| { 

            match absolute_path(Path::new(str)) {
                Ok(ret) => {

                    if Path::new(&ret).is_file() {                       
                        files.push(ListerFile {
                            useconf: false,
                            path: OsString::from(ret)
                        });
                    } else {
                        warn!(
                        "Cannot find file!",
                        "
                            File given via. cmd-line arguments...
                                {}
                            ...with fullpath of...
                                {}
                            Does not exist!
                        ",
                            fmterr_val!(str),
                            fmterr_val!(ret.display())
                        )
                    }

                }

                _ => {}
            }

        });

    }

    if files.len() == 0{

        reterr!(Error::NoFilesFound)

    } else {

        // Removes duped.
        Ok(files.into_iter().unique_by(|x|x.path.clone()).collect_vec())

    } 

}
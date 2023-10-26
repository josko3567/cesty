use crate::{
    config::Config, 
    argument::Argument,
    error::ErrorGroup
};

use std::{
    error::Error, 
    fmt::Display, 
    ffi::OsString, 
    path::{Path, PathBuf}
};

use globwalk;
use itertools::Itertools;
use path_clean::PathClean;
use indoc::indoc;
use colored::Colorize;

// const ERROR_GROUP: u32 = 3;

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum ListerError {
    NoFilesFound,
    FilesystemOperationFail,
}

impl ListerError {

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

impl Display for ListerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match &self {
            Self::NoFilesFound => {
                format!( indoc!{"
                Error!
                    No files given to parse.
                    Not from the config file.
                    Not from the command line arguments.
                "})
                .red()},
            Self::FilesystemOperationFail => {
                format!( indoc!{"
                Error!
                    Filesystem operation failed.
                "})
                .red()}
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

impl Error for ListerError {}

#[derive(Hash, PartialEq, Clone)]
pub struct ListerFile {

    pub useconf: bool,
    pub path: OsString

}

impl ListerFile {

    pub fn new() -> ListerFile {

        ListerFile {
            useconf: true,
            path: OsString::from("")
        }

    }

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

    let mut cln = fullpath.clone();
    let mut solved = false;

    loop {
        if !cln.pop() {break;}
        if cln.is_dir() {
            solved = true;
            break;
        }
    }

    if solved == false {

        return None;

    }
    
    let str = cln.to_string_lossy().to_string();
    let filestr: String = fullpath.to_string_lossy().to_string();
    // println!("{} + {}", str, &filestr[str.len()..filestr.len()]);
    Some(filestr[str.len()..filestr.len()].matches(&['/', '\\']).count())

}


#[allow(dead_code)]
fn absolute_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {

    let path = path.as_ref();

    let absolute_path = if path.is_absolute() {

        path.to_path_buf()

    } else {

        std::env::current_dir()?.join(path)

    }
    .clean();

    Ok(absolute_path)

}

/// Get a vector of ListerFiles that contain full paths of files to be
/// processed and what compiler options to use with them.
#[allow(non_snake_case)]
pub fn get_list(conf: &Config, args: &Vec<Argument>) -> Result<Vec<ListerFile>, ListerError> {

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
        _ => {&binding_string_empty}

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

    let mut files = Vec::<ListerFile>::new();

    if arg_recipe.is_some() {

        let unwrap_recipe = arg_recipe.unwrap();
        
        for run in unwrap_recipe.run.iter() {
            
            let recurse = run.recurse.is_some_and(|x|x);
            let symlink = run.symlinks.is_some_and(|x|x);

            // Fullpath of the recipe.run[?].path variable
            let mut fullpath= if Path::new(&run.path).is_absolute() {
                PathBuf::from(&run.path)
            } else {
                let mut confpath = PathBuf::from(&conf.path);

                // Removes .cesty.{conf,yaml,yml}
                confpath.pop();

                // Pushes local dir/file reference
                confpath.push(&run.path);

                match Path::new(&confpath).is_absolute() {
                    true => {confpath}
                    _ => { 
                        eprintln!("{} {}", 
                            "From lister.rs\n".dimmed(),
                            format!( indoc!{"
                            Warning! 
                                Failed to extract full path from:
                                    Path: {}
                                Reached fullpath value of:
                                    Fullpath: {}
                            "}, 
                                &run.path, 
                                confpath.display()
                            ).yellow()
                        );
                        break;
                    }
                }
            };
            
            // Check if path points to file... 
            if fullpath.is_file() {
                files.append(&mut vec![ListerFile{
                    useconf: true,
                    path: fullpath.as_os_str().to_os_string()
                }]);
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
                        eprintln!("{} {}", 
                            "From lister.rs\n".dimmed(),
                            format!( indoc!{"
                            Warning! 
                                Failed to extract max depth from:
                                    Path: {}
                                    Fullpath: {}
                            "}, 
                                &run.path.to_string().underline(), 
                                fullpath.to_string_lossy()
                                    .to_string().underline()
                            ).yellow()
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
                Ok(ret) => {ret}
                Err(_err) => {
                    eprintln!("{} {}", 
                        "From lister.rs\n".dimmed(),
                        format!( indoc!{"
                        Warning! 
                            Failed to open a file in:
                                Path: {}
                                Fullpath: {}
                            Due to:
                                Error: {}
                        "}, 
                            &run.path.to_string().underline(), 
                            fullpath.display().to_string().underline(), 
                            _err.to_string().red()
                        ).yellow()
                    );
                    continue;
                }   

            }
            .for_each(|x| { match x {
                Ok(res) => { 
                // println!("{:?}", res.path().as_os_str().to_os_string());
                    if res.path().is_file() {
                        files.append(&mut vec![ListerFile{
                            useconf: true,
                            path: res.path().as_os_str().to_os_string()
                        }])
                    }
                },
                Err(_err) => {
                    eprintln!("{} {}", 
                        "From lister.rs\n".dimmed(),
                        format!( indoc!{"
                        Warning! 
                            Failed to open a file in:
                                Path: {}
                                Fullpath: {}
                            Due to:
                                Error: {}
                        "}, 
                            &run.path.to_string().underline(), 
                            fullpath.display().to_string().underline(), 
                            _err.to_string().red()
                        ).yellow()
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
                        files.append(&mut vec![ListerFile {
                            useconf: false,
                            path: OsString::from(ret)
                        }]);
                    } else {
                        eprintln!("{} {}",
                            "From lister.rs\n".dimmed(),
                            format!( indoc!{"
                            Warning! 
                                Argument file:
                                    Path: \"{}\"
                                    Fullpath: \"{}\"
                                Does not exist!
                            "}, 
                                &str.underline(), 
                                ret.display().to_string().underline()
                            ).yellow()
                        );
                    }

                }

                _ => {}
            }

        });

    }

    if files.len() == 0{

        Err(ListerError::NoFilesFound)

    } else {

        // Removes duped.
        Ok(files.into_iter().unique_by(|x|x.path.clone()).collect_vec())

    } 

}
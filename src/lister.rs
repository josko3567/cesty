//! Enlists all possible files to parse for tests.

use std::path::PathBuf;

use globwalk::{
    glob_builder, DirEntry, 
    FileType, WalkError
};

use path_clean::PathClean;

use crate::{
    arg_conf::{
        Recipe, Run
    }, 
    defaults::{
        get_max_depth, 
        SEARCHED_FILE_EXTENSION
    }, 
    error::{
        debuginfo, error, function_message, warning, Alert, AlertInfo
    }
};

/// List all files with the supported [`SEARCHED_FILE_EXTENSION`].
/// 
/// Paths that are parsed are:
///     - CLI passed paths found in [Run::files]
///     - [Recipe::parse_path]'s from the [Recipe] whose 
///       [Recipe::name] is the same as [Run::recipe_name].
/// 
/// Files
/// -----
/// If a path is found out to be a file, its cleaned up and included into the returning
/// [Vec<PathBuf>].
/// 
/// Directories
/// -----------
/// If a path is found out to be a directory, its contents are parsed for files with
/// the supported [`SEARCHED_FILE_EXTENSION`].
/// 
/// Globs
/// -----
/// Globs are supported and work as intended.
pub fn list(run_conf: &Run) -> Result<((Vec<PathBuf>, Option<&Recipe>), Vec<Alert>), Alert> {

    let mut warnings: Vec<Alert>   = vec![];
    let mut paths:    Vec<PathBuf> = vec![];

    // Append all arguments files into the paths variable.
    paths.append(
        &mut run_conf.files
            .iter()
            .filter(|path| 
                if path.is_file()
                && path.extension().is_some_and(
                    |ex| SEARCHED_FILE_EXTENSION.iter().find(
                        |searched| &&ex == searched).is_some()) 
                {
                    true
                } 
                else 
                {
                    false
                }
            )
            .map(|x| x.to_owned())
            .collect()
    );


    // Checks if the recipe name is specified and pass only run_conf.files
    if run_conf.recipe_name.is_none() {

        if paths.is_empty() {

            return error!{
                debug: debuginfo!(),
                description: "no files found to parse".to_owned(),
                example: None,
                note: vec![
                    "specify a recipe or pass files through the command line with the -F / --files option.".to_owned()
                ]
            }
            
        }

        if run_conf.no_config == false {

            warnings.push(warning!{
                debug: debuginfo!(),
                description: "no recipe name specified".to_owned(),
                example: None,
                note: vec![
                    "using only files passed through the -F / --files argument".to_owned()
                ]
            })

        }

        return Ok((({paths.sort(); paths.dedup(); paths}, None), warnings));

    }
    
    let recipe = match run_conf.recipes
        .iter()
        .find(|x| &x.name == run_conf.recipe_name.as_ref().unwrap())
    {
        Some(recipe) => recipe,
        None => return error!{
            debug: debuginfo!(),
            description: format!("recipe `{}` was not found", run_conf.recipe_name.as_ref().unwrap()),
            example: None,
            note: if run_conf.no_config == true {
                vec![
                    "cesty did not attempt to find a config because the -n / --noconfig flag was set.".to_owned(),
                    format!("due to no config being found no recipe with the name `{}` was found.",
                        run_conf.recipe_name.as_ref().unwrap()),
                    format!("remove the -n / --noconfig flag or remove `{}` recipe name from your CLI arguments.", 
                        run_conf.recipe_name.as_ref().unwrap())
                ]
            }
            else {  
                vec![
                    format!("a config file may have not been found therefore no recipe named `{}` was found.", 
                        run_conf.recipe_name.as_ref().unwrap()),
                    "check all prior warnings to see if this is true.".to_owned()
                ]
            }
        }
    };

    for parse_path in recipe.parse_path.iter() {

        // Check if path is a file, if so push it to paths
        if parse_path.path.is_file() {
            paths.push(parse_path.path.clean());
            break;
        }

        let (path, path_buf) = {

            let mut temp_path = parse_path.path.clean();

            if temp_path.is_dir() 
            || temp_path.file_name().is_some_and(|x| x == "*") {

                temp_path.push(format!("*.{{{}}}", SEARCHED_FILE_EXTENSION.join(",")));
            
            }

            let complete_path = if !temp_path.is_absolute() {

                let mut absolute_path = run_conf.config_path.clone()
                    .expect("Unreachable, config path was None even though it was parsed.");
                absolute_path.pop();
                absolute_path.push(temp_path);
                absolute_path
                
            } else {
                
                temp_path
                
            }.clean();
            
            (complete_path.to_str().expect("Failed to convert PathBuf to &str").to_string(), complete_path)
        
        };

        let depth = if parse_path.recursive.is_some_and(|x| x == true) {
            let max = get_max_depth(&path_buf);
            (max, usize::MAX)
        } else {
            let max = get_max_depth(&path_buf);
            (max, max)
        };


        // TODO:
        // This is garbage, as it requires a conversion from PathBuf -> &str which
        // could fail for some operating systems (as it says in [PathBuf::to_str])
        // glob_builder itself converts &str -> PathBuf so this is just a
        // unnecessary conversion that could fail.
        let walker = match glob_builder(&path)
            .case_insensitive(false)
            .contents_first(true)
            .file_type(FileType::FILE)
            .max_depth(depth.1)
            .min_depth(depth.0)
            .follow_links(false)
            .build()
        {
            Ok(gw) => gw,
            Err(globerr) => {
                return error!{
                    debug: debuginfo!(),
                    description: format!("issue occurred while trying to parse path `{path}` for C files"),
                    example: None,
                    note: function_message!("glowalk::glob_builder().build()", globerr.to_string())
                }
            }
        };

        let file_iter: Vec<Result<DirEntry, WalkError>> = walker.collect();
        
        for result in file_iter {

            let entry = match result {

                Ok(entry) => PathBuf::from(entry.path()),
                Err(err) => {
                    warnings.push(warning!{
                        description: format!("a error was returned while traversing `{path}` for C files"),
                        debug: debuginfo!(),
                        example: None,
                        note: vec![
                            "the error message returned is the following:".to_owned(),
                            err.to_string()
                        ]
                    });
                    continue;
                }

            };

            if entry.is_file()
            && entry.extension().is_some_and(
                |file_extension| SEARCHED_FILE_EXTENSION.iter().find(
                    |searched_for_extension| &&file_extension == searched_for_extension).is_some()) 
            {
                paths.push(entry)
            }

        }

    }

    Ok((({paths.sort(); paths.dedup(); paths}, Some(recipe)), warnings))    

}
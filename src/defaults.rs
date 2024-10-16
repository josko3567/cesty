//! Default values and common functions used across cesty-s
//! source code.

use std::path::PathBuf;

/// File extensions that can be parsed by [crate::extract::extract].
/// 
/// File extensions that are returned from [crate::lister::list].
pub const SEARCHED_FILE_EXTENSION: [&'static str; 1] = ["c"];

/// Second part of a config name.
/// 
/// Example
/// -------
/// config.<ins>cesty</ins>.[[toml],[yaml](serde_yaml)]
pub const CONFIG_FOLLOWUP_NAME: &'static str = "cesty";

/// Default name for configs created with `cesty init <yaml/toml>`.
pub const DEFAULT_CONFIG_FILENAME: &'static str = "config";

/// Private directory used by cesty, kind of like `.git`.
pub const DEFAULT_PRIVATE_DIRECTORY: &'static str = ".cesty";

/// Function prefix to detect what function is used for testing.
/// 
/// Example
/// -------
/// Valid tests.
/// ```C
/// int cesty_addition_test(int a, int b) { ... }
/// int cesty_image_compression_test(struct bitmap image) { ... }
/// ```
pub const DEFAULT_FUNCTION_PREFIX: &'static str = "cesty_";

/// Name of the default compiler when no compiler
/// is specified.
pub const DEFAULT_COMPILER_NAME: &'static str = "gcc";

/// Amount of attempts at creating a unique directory name
pub const MAX_BATCH_ROOT_NAME_CREATION_ATTEMPTS: usize = 10;

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
pub fn get_max_depth(fullpath: &PathBuf) -> usize {

    let mut clone = fullpath.clone();
    let mut pops: usize = 0;

    loop {
        if  clone.is_dir()
        || !clone.pop() 
        {
            break
        }
        pops += 1;
    }

    return pops;

}
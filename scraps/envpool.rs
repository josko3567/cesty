use lazy_static::lazy_static;
use std::{collections::HashMap, sync::Mutex};
use crate::environment::Environments;

lazy_static!(
    /// ### pool of file environments.
    /// 
    /// Environments can depend on standalone switch,
    /// therefore we have environment.full which is 
    /// a copy of the file and environment.bodyclean
    /// which is a copy of the file without function bodies.
    pub static ref POOL: Mutex<HashMap<&'static str, Environments>> = {
        let a = HashMap::new();
        Mutex::new(a)
    };
);


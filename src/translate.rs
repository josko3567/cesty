use crate::{extract::*, environment::Environment, config::Config};


struct TranslatedFiles {

    arg: String,
    test: String

}

impl TranslatedFiles {

    fn from_multi(
        input: &Extract, 
        env:   &Environment, 
        conf:  &Config
    ) -> Vec<TranslatedFiles> {

        return vec![];

    }

    fn from_single(
        input: &Test, 
        env:   &Environment,
        conf:  &Config
    ) -> TranslatedFiles {

        return TranslatedFiles {
            arg: String::new(),
            test: String::new()
        };

    }

}

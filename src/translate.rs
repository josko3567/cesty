use crate::extract::*;


struct TranslatedFiles {

    arg: String,
    test: String

}

impl TranslatedFiles {

    fn from_multi(input: &Extract) -> Vec<TranslatedFiles> {

        return vec![];

    }

    fn from_single(input: &ExtractTest, file: &String) -> TranslatedFiles {

        return TranslatedFiles {
            arg: String::new(),
            test: String::new()
        };

    }

}

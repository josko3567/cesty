use strum_macros::EnumString;

#[derive(EnumString)]
#[repr(u8)]
enum Replaceable {
    
    FOLDER(String),
    FILENAME(String),
    FILE(String)

}


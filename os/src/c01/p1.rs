use std::{fs, io::Error};

fn main() -> Result<(), Error> {
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        println!("{}", if let Ok(t) = entry.file_name().into_string() { t } else { "Failed".to_string() });
    }
    Ok(())
}
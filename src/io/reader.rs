use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::Model;

pub fn read_model(input_path: impl AsRef<Path>) -> io::Result<Model> {
    Ok(serde_json::from_reader(BufReader::new(File::open(
        input_path,
    )?))?)
}

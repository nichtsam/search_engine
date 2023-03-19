use std::{
    fs::File,
    io::{self, BufReader},
    path::Path,
};

use crate::DocumentTermsFrequenciesIndex;

pub fn read_index(path: impl AsRef<Path>) -> io::Result<DocumentTermsFrequenciesIndex> {
    Ok(serde_json::from_reader(BufReader::new(File::open(path)?))?)
}

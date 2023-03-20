use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
};

use crate::Model;

pub fn write_model(model: &Model, output_path: impl AsRef<Path>) -> io::Result<()> {
    serde_json::to_writer(BufWriter::new(File::create(output_path)?), model)?;
    Ok(())
}

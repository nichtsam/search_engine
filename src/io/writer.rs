use std::{
    fs::File,
    io::{self, BufWriter},
    path::Path,
};

use crate::DocumentTermsFrequenciesIndex;

pub fn save_index(
    index: &DocumentTermsFrequenciesIndex,
    output_path: impl AsRef<Path>,
) -> io::Result<()> {
    serde_json::to_writer(BufWriter::new(File::create(output_path)?), &index)?;
    Ok(())
}

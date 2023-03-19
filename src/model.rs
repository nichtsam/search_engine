use std::{collections::HashMap, path::PathBuf};

pub type DocumentTermsFrequencies = HashMap<String, usize>;
pub type DocumentTermsFrequenciesIndex = HashMap<PathBuf, DocumentTermsFrequencies>;

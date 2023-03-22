use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

use crate::lexer::Lexer;

pub type DocumentTermsFrequencies = HashMap<String, usize>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Doc {
    pub dtf: DocumentTermsFrequencies,
    pub dtc: usize,
}

pub type DocumentIndex = HashMap<PathBuf, Doc>;
pub type TermCorpuswideFrequencyTable = HashMap<String, usize>;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Model {
    pub doc_index: DocumentIndex,
    pub tcf_table: TermCorpuswideFrequencyTable,
}

use indexer::*;
use searcher::*;
impl Model {
    pub fn add_documents(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let dir = fs::read_dir(path)?;

        'next_file: for entry in dir {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(err) => {
                    eprintln!("ERROR: {err}");
                    continue 'next_file;
                }
            };

            println!("indexing {path:?}...");

            if path.is_dir() {
                if let Err(err) = self.add_documents(path) {
                    eprintln!("ERROR: {err}");
                }

                continue 'next_file;
            }

            match path.extension().and_then(|os_str| os_str.to_str()) {
                Some(extension) => match extension {
                    "html" => {
                        let text = match extract_text_from_html_file(&path) {
                            Some(v) => v,
                            None => {
                                eprintln!("ERROR: could not extract text from path: {path:?}");
                                continue 'next_file;
                            }
                        };

                        let doc = index_text(&text);
                        for term in doc.dtf.keys() {
                            *self.tcf_table.entry(term.to_string()).or_insert(0) += 1;
                        }
                        self.doc_index.insert(path, doc);
                    }
                    other => {
                        eprintln!("ERROR: extension \"{other}\" is not supported");
                        continue 'next_file;
                    }
                },
                None => {
                    eprintln!("ERROR: recursive indexing is not supported");
                    continue 'next_file;
                }
            }
        }

        Ok(())
    }

    pub fn search(&self, keyword_phrase: &str) -> Vec<(&PathBuf, f32)> {
        compute_search(keyword_phrase, self)
    }
}

mod indexer {
    use super::*;

    pub fn index_text(text: &str) -> Doc {
        let chars = &text.chars().collect::<Vec<_>>();

        let mut dtf: DocumentTermsFrequencies = HashMap::new();
        let mut dtc = 0;

        for token in Lexer::new(chars) {
            let count = dtf.entry(token).or_insert(0);
            *count += 1;
            dtc += 1;
        }

        Doc { dtf, dtc }
    }

    pub fn extract_text_from_html_file(file_path: impl AsRef<Path>) -> Option<String> {
        let content = fs::read_to_string(file_path).ok()?;
        extract_text_from_html(&content)
    }

    pub fn extract_text_from_html(html: &str) -> Option<String> {
        // parse the HTML
        // let html = r#"<html><title>This is The Title</title><script>this should not be included</script></html>"#;
        let document = Html::parse_document(html);

        // select all text nodes
        let selector =
            Selector::parse("body :not(script):not(style), head :not(script):not(style)").ok()?;

        Some(
            document
                .select(&selector)
                .flat_map(|tag| tag.text())
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

mod searcher {
    use super::*;

    pub fn compute_search<'a>(keyword_phrase: &str, model: &'a Model) -> Vec<(&'a PathBuf, f32)> {
        let keyword_phrase = &keyword_phrase.chars().collect::<Vec<_>>();
        let mut result = Vec::new();
        for (path, doc) in model.doc_index.iter() {
            let lexer = Lexer::new(keyword_phrase);
            let mut rank_score = 0.0;

            for token in lexer {
                rank_score += compute_tf(&token, doc) * compute_idf(&token, model);
            }

            result.push((path, rank_score));
        }

        result.sort_by(|(_, rank_score_a), (_, rank_score_b)| {
            rank_score_a
                .partial_cmp(rank_score_b)
                .unwrap_or(Ordering::Equal)
        });
        result.reverse();

        result
    }

    pub fn compute_tf(term: &str, doc: &Doc) -> f32 {
        // where
        // s => sum of terms
        // t => frequency of term
        let Doc { dtf, dtc } = doc;

        let s = *dtc;
        let t = *dtf.get(term).unwrap_or(&0);

        t as f32 / s as f32
    }

    pub fn compute_idf(term: &str, model: &Model) -> f32 {
        // where
        // n => total number of documents in the corpus
        // d => number of documents where the term appears
        let Model {
            doc_index,
            tcf_table,
        } = model;

        let n = doc_index.len();
        let d = *tcf_table.get(term).unwrap_or(&0);

        ((1 + n) as f32 / (1 + d) as f32).log10()
    }
}

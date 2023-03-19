use std::{collections::HashMap, fs, io, path::Path};

use scraper::{Html, Selector};

use crate::{lexer::Lexer, DocumentTermsFrequencies, DocumentTermsFrequenciesIndex};

pub fn index_dir(
    dir_path: impl AsRef<Path>,
    dtf_index: &mut DocumentTermsFrequenciesIndex,
) -> io::Result<()> {
    let dir = fs::read_dir(dir_path)?;

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
            if let Err(err) = index_dir(path, dtf_index) {
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

                    let dtf = index_text(&text);
                    dtf_index.insert(path, dtf);
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

pub fn index_text(text: &str) -> DocumentTermsFrequencies {
    let chars = &text.chars().collect::<Vec<_>>();

    let mut dtf: DocumentTermsFrequencies = HashMap::new();

    for token in Lexer::new(chars) {
        let count = dtf.entry(token).or_insert(0);
        *count += 1;
    }

    dtf
}

fn extract_text_from_html_file(file_path: impl AsRef<Path>) -> Option<String> {
    let content = fs::read_to_string(file_path).ok()?;
    extract_text_from_html(&content)
}

fn extract_text_from_html(html: &str) -> Option<String> {
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

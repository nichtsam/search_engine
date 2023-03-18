use clap::{CommandFactory, Parser, Subcommand};
use scraper::{Html, Selector};
use std::{
    cmp::Ordering,
    collections::HashMap,
    fs::{self, read_dir, File},
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
struct Lexer<'a> {
    content: &'a [char],
}

impl<'a> Lexer<'a> {
    fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while self.content.len() > 0 && self.content[0].is_whitespace() {
            self.content = &self.content[1..];
        }
    }

    fn chop(&mut self, length: usize) -> &'a [char] {
        let token = &self.content[..length];
        self.content = &self.content[length..];
        token
    }

    fn chop_while<P>(&mut self, predicate: P) -> &'a [char]
    where
        P: Fn(&char) -> bool,
    {
        let mut n = 0;
        while n < self.content.len() && predicate(&self.content[n]) {
            n += 1;
        }

        self.chop(n)
    }

    fn next_token(&mut self) -> Option<String> {
        self.trim_left();

        // reach end
        if self.content.len() == 0 {
            return None;
        }

        // token starts with number
        if self.content[0].is_numeric() {
            let token = self.chop_while(|c| c.is_numeric()).iter().collect();
            return Some(token);
        }
        // token starts with alphabet
        if self.content[0].is_alphabetic() {
            let token = self
                .chop_while(|c| c.is_alphanumeric())
                .iter()
                .map(|c| c.to_ascii_uppercase())
                .collect();
            return Some(token);
        }

        // token starts with symbols
        let token = self.chop(1).iter().collect();
        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

type DocumentTermsFrequencies = HashMap<String, usize>;
type DocumentTermsFrequenciesIndex = HashMap<PathBuf, DocumentTermsFrequencies>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "index the specified path recursively")]
    Index {
        #[arg(help = "the directory of the collections you want to index.")]
        input_dir: String,
        #[arg(help = "the path to output the index result for further usage.")]
        output_path: String,
    },
    #[command(about = "lists out top 10 most relevant document")]
    Search {
        #[arg(help = "the word or phrase that you’d like to rank for.")]
        keyword_phrase: String,
        #[arg(help = "the path of the document terms frequencies index to search the term in.")]
        dtf_index_path: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index {
            input_dir,
            output_path,
        } => {
            let mut dtf_index = DocumentTermsFrequenciesIndex::new();

            if let Err(err) = index_dir(input_dir, &mut dtf_index) {
                let mut cmd = Cli::command();
                cmd.error(clap::error::ErrorKind::Io, err).exit();
            };

            if let Err(err) = save_index(&dtf_index, output_path) {
                eprintln!("ERROR: could not save index to path {output_path}: {err}");
            }
        }

        Commands::Search {
            keyword_phrase,
            dtf_index_path,
        } => {
            let dtf_index_file = File::open(dtf_index_path).unwrap_or_else(|err| {
                let mut cmd = Cli::command();
                cmd.error(clap::error::ErrorKind::Io, err).exit();
            });

            let dtf_index: DocumentTermsFrequenciesIndex = serde_json::from_reader(dtf_index_file)
                .unwrap_or_else(|err| {
                    let mut cmd = Cli::command();
                    cmd.error(clap::error::ErrorKind::Io, err).exit();
                });

            search(keyword_phrase, &dtf_index);
        }
    }
}

fn search(keyword_phrase: &str, dtf_index: &DocumentTermsFrequenciesIndex) {
    let keyword_phrase = &keyword_phrase.chars().collect::<Vec<_>>();
    let mut result = Vec::new();
    for (path, dtf) in dtf_index {
        let lexer = Lexer::new(keyword_phrase);
        let mut rank_score = 0.0;

        for token in lexer {
            rank_score += compute_tf(&token, dtf) * compute_idf(&token, dtf_index);
        }

        result.push((path, rank_score));
    }

    result.sort_by(|(_, rank_score_a), (_, rank_score_b)| {
        rank_score_a
            .partial_cmp(rank_score_b)
            .unwrap_or(Ordering::Equal)
    });
    result.reverse();

    for (index, (path, rank_score)) in result.iter().enumerate().take(10) {
        println!(
            "{no}. {path} => {rank_score}",
            no = index + 1,
            path = path.display()
        );
    }
}

fn compute_tf(term: &str, dtf: &DocumentTermsFrequencies) -> f32 {
    // where
    // s => sum of terms
    // t => frequency of term
    let s = dtf.iter().map(|(_, f)| *f).sum::<usize>();
    let t = *dtf.get(term).unwrap_or(&0);

    t as f32 / s as f32
}

fn compute_idf(term: &str, dtf_index: &DocumentTermsFrequenciesIndex) -> f32 {
    // where
    // n => total number of documents in the corpus
    // d => number of documents where the term appears
    let n = dtf_index.len();
    let d = dtf_index
        .iter()
        .filter(|(_, dtf)| dtf.contains_key(term))
        .collect::<Vec<_>>()
        .len();

    (n as f32 / (1 + d) as f32).log10()
}

fn index_dir(
    dir_path: impl AsRef<Path>,
    dtf_index: &mut DocumentTermsFrequenciesIndex,
) -> io::Result<()> {
    let dir = read_dir(dir_path)?;

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

        match path.extension().map(|os_str| os_str.to_str()).flatten() {
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

fn save_index(
    dtf_index: &DocumentTermsFrequenciesIndex,
    output_path: impl AsRef<Path>,
) -> io::Result<()> {
    serde_json::to_writer(File::create(output_path)?, &dtf_index)?;

    Ok(())
}

fn index_text(text: &str) -> DocumentTermsFrequencies {
    let chars = &text.chars().collect::<Vec<_>>();

    let mut dtf: DocumentTermsFrequencies = HashMap::new();

    for token in Lexer::new(&chars) {
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
            .map(|tag| tag.text())
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    )
}

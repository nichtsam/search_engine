use clap::{CommandFactory, Parser, Subcommand};
use scraper::{Html, Selector};
use std::{
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

type TermFrequency = HashMap<String, usize>;
type TermFrequencyIndex = HashMap<PathBuf, TermFrequency>;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "index the specified path recursively")]
    Index { dir_path: String },
    #[command(about = "currently only check how many files is indexed")]
    Search,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Index { dir_path } => {
            let mut term_frequency_index = TermFrequencyIndex::new();

            if let Err(err) = index_dir(dir_path, &mut term_frequency_index) {
                let mut cmd = Cli::command();
                cmd.error(clap::error::ErrorKind::Io, err).exit();
            };
        }
        Commands::Search => {
            let index_file = File::open("term_frequency_index.json").unwrap_or_else(|err| {
                let mut cmd = Cli::command();
                cmd.error(clap::error::ErrorKind::Io, err).exit();
            });

            let term_frequency_index: TermFrequencyIndex = serde_json::from_reader(index_file)
                .unwrap_or_else(|err| {
                    let mut cmd = Cli::command();
                    cmd.error(clap::error::ErrorKind::Io, err).exit();
                });

            println!("index contains {} files", term_frequency_index.len());
        }
    }
}

fn index_dir(
    dir_path: impl AsRef<Path>,
    term_frequency_index: &mut TermFrequencyIndex,
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
            if let Err(err) = index_dir(path, term_frequency_index) {
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

                    let term_frequency = index_text(&text);
                    term_frequency_index.insert(path, term_frequency);
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

    save_index(&term_frequency_index, "term_frequency_index.json")?;

    Ok(())
}

fn save_index(
    term_frequency_index: &TermFrequencyIndex,
    index_path: impl AsRef<Path>,
) -> io::Result<()> {
    serde_json::to_writer(File::create(index_path)?, &term_frequency_index)?;

    Ok(())
}

fn index_text(text: &str) -> TermFrequency {
    let chars = &text.chars().collect::<Vec<_>>();

    let mut term_frequency: TermFrequency = HashMap::new();

    for token in Lexer::new(&chars) {
        let count = term_frequency.entry(token).or_insert(0);
        *count += 1;
    }

    term_frequency
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

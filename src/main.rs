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

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        // reach end
        if self.content.len() == 0 {
            return None;
        }

        // token starts with number
        if self.content[0].is_numeric() {
            let token = self.chop_while(|c| c.is_numeric());
            return Some(token);
        }
        // token starts with alphabet
        if self.content[0].is_alphabetic() {
            let token = self.chop_while(|c| c.is_alphanumeric());
            return Some(token);
        }

        // token starts with symbols
        let token = self.chop(1);
        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

type TermFrequency = HashMap<String, usize>;
type TermFrequencyIndex = HashMap<PathBuf, TermFrequency>;

fn main() -> io::Result<()> {
    let dir_path = "path/to/folder/";
    let dir = read_dir(dir_path)?;

    let mut term_frequency_index = TermFrequencyIndex::new();

    for file in dir {
        let file_path = file?.path();
        println!("indexing {file_path:?}...");

        let text = extract_text_from_html_file(&file_path)?;
        let chars = &text
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .collect::<Vec<_>>();

        let mut term_frequency: TermFrequency = HashMap::new();

        for token in Lexer::new(&chars) {
            let term = token.iter().collect::<String>();
            let count = term_frequency.entry(term).or_insert(0);
            *count += 1;
        }

        term_frequency_index.insert(file_path, term_frequency);
    }

    let buffer = File::create("term_frequency_index.json")?;
    serde_json::to_writer(buffer, &term_frequency_index)?;

    Ok(())
}

fn extract_text_from_html_file(file_path: impl AsRef<Path>) -> io::Result<String> {
    let content = fs::read_to_string(file_path)?;
    Ok(extract_text_from_html(&content))
}

fn extract_text_from_html(html: &str) -> String {
    // parse the HTML
    // let html = r#"<html><title>This is The Title</title><script>this should not be included</script></html>"#;
    let document = Html::parse_document(html);

    // select all text nodes
    let selector = Selector::parse("html").unwrap();

    document
        .select(&selector)
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
}

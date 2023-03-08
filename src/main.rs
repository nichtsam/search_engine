use scraper::{Html, Selector};
use std::{
    fs::{self, read_dir},
    io,
    path::Path,
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

    fn next_token(&mut self) -> Option<&'a [char]> {
        self.trim_left();

        // reach end
        if self.content.len() == 0 {
            return None;
        }

        // token starts with number
        if self.content[0].is_numeric() {
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_numeric() {
                n += 1;
            }

            let token = &self.content[..n];
            self.content = &self.content[n..];

            return Some(token);
        }
        // token starts with alphabet
        if self.content[0].is_alphabetic() {
            let mut n = 0;
            while n < self.content.len() && self.content[n].is_alphanumeric() {
                n += 1;
            }

            let token = &self.content[..n];
            self.content = &self.content[n..];

            return Some(token);
        }

        // token starts with symbols
        let token = &self.content[..1];
        self.content = &self.content[1..];

        Some(token)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = &'a [char];

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn main() -> io::Result<()> {
    let dir_path = "path/to/folder/";
    let dir = read_dir(dir_path)?;

    for file in dir {
        let file_path = file?.path();
        let text = extract_text_from_html_file(&file_path)?;
        let chars = &text
            .chars()
            .map(|c| c.to_ascii_uppercase())
            .collect::<Vec<_>>();

        let lexer = Lexer::new(&chars);
        for token in lexer {
            println!("{}", token.iter().collect::<String>());
        }
    }

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
        .filter_map(|t| {
            let t = t.trim();
            if t.is_empty() {
                return None;
            };
            Some(t)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

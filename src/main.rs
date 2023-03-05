use scraper::{Html, Selector};
use std::{
    fs::{self, read_dir},
    io,
    path::Path,
};

fn main() -> io::Result<()> {
    let dir_path = "path/to/folder/";
    let dir = read_dir(dir_path)?;

    for file in dir {
        let file_path = file?.path();
        let text = extract_text_from_html_file(&file_path).unwrap();

        println!("{file_path:?} {size}", size = text.len());
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

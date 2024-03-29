use rust_stemmers::Stemmer;

#[derive(Debug)]
pub struct Tokenizer<'a> {
    content: &'a [char],
}

impl<'a> Tokenizer<'a> {
    pub fn new(content: &'a [char]) -> Self {
        Self { content }
    }

    fn trim_left(&mut self) {
        while !self.content.is_empty() && self.content[0].is_whitespace() {
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
        if self.content.is_empty() {
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
                .map(|c| c.to_ascii_lowercase())
                .collect::<String>();
            let stemmer = Stemmer::create(rust_stemmers::Algorithm::English);
            let stemmed = stemmer.stem(&token).to_string();
            return Some(stemmed);
        }

        // token starts with symbols
        let token = self.chop(1).iter().collect();
        Some(token)
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

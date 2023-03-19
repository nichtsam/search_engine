use std::{cmp::Ordering, path::PathBuf};

use crate::{
    lexer::Lexer,
    model::{DocumentTermsFrequencies, DocumentTermsFrequenciesIndex},
};

pub fn search(keyword_phrase: &str, dtf_index: &DocumentTermsFrequenciesIndex) {
    let result = compute_search(keyword_phrase, dtf_index);

    for (index, (path, rank_score)) in result.iter().enumerate().take(10) {
        println!(
            "{no}. {path} => {rank_score}",
            no = index + 1,
            path = path.display()
        );
    }
}

fn compute_search<'a>(
    keyword_phrase: &str,
    dtf_index: &'a DocumentTermsFrequenciesIndex,
) -> Vec<(&'a PathBuf, f32)> {
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

    result
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
        .count();

    ((1 + n) as f32 / (1 + d) as f32).log10()
}

#![allow(dead_code)]

use std::collections::HashMap;

use regex::Regex;
use rayon::prelude::*;

use fuzzywuzzy;
use savefile::prelude::*;
use savefile_derive::Savefile;
use unidecode::unidecode;

#[derive(Debug, Clone, Savefile)]
pub struct Category {
    pub parent: String,
    pub category: String,
    pub url: String,
}

impl ToString for Category {
    fn to_string(&self) -> String {
        self.parent.to_string() + " " + &self.category
    }
}

#[derive(Debug, Clone)]
struct Token {
    word: String,
    vocab_index: usize,
    weight: f32,
}

const MAX_TOKEN_NEIGHBOUR_WINDOW: usize = 4;

fn get_token_neighbours(tokens: &Vec<Token>, token_pos: usize) -> Vec<Token> {
    let neighbour_start_index =
        (token_pos as i32 - MAX_TOKEN_NEIGHBOUR_WINDOW as i32).max(0) as usize;
    let neighbour_end_index = (token_pos + MAX_TOKEN_NEIGHBOUR_WINDOW).min(tokens.len() - 1);

    let left_neighbours = &tokens[neighbour_start_index..token_pos];
    let right_neighbours = &tokens[(token_pos + 1)..=neighbour_end_index];

    let mut neighbours = Vec::with_capacity(left_neighbours.len() + right_neighbours.len());
    left_neighbours
        .iter()
        .for_each(|neighbour| neighbours.push(neighbour.clone()));
    right_neighbours
        .iter()
        .for_each(|neighbour| neighbours.push(neighbour.clone()));

    neighbours
}

fn find_most_similar_word_in_vocab(
    word: &str,
    vocab: &Vec<String>,
    threshold: u8,
) -> Option<(usize, String, f32)> {
    let result = vocab
        .par_iter()
        .enumerate()
        .filter_map(|(vocab_index, vocab_token_word)| {
            let mut ratio = fuzzywuzzy::fuzz::ratio(word, &vocab_token_word);
            if word.len() != vocab_token_word.len()
                || vocab_token_word.starts_with("-")
                || vocab_token_word.starts_with("=")
                || vocab_token_word.starts_with("(")
                || vocab_token_word.starts_with(")")
            {
                ratio = (ratio as i8 - 5).max(0) as u8;
            }
            if ratio >= threshold {
                Some((
                    vocab_index,
                    vocab_token_word.to_string(),
                    ratio as f32,
                ))
            } else {
                None
            }
        })
        .reduce(|| (0usize, "".to_string(), 0f32), |largest, current| {
            if largest.2 < current.2 {
                current.clone()
            } else {
                largest
            }
        });
    if result.1 == "" {
        None
    } else {
        Some((result.0, result.1.to_string(), result.2 / 100f32))
    }
}

fn predict(input: &str, vocab: &Vec<String>, categories: &Vec<&str>) -> HashMap<String, f32> {
    let regex = Regex::new(r"\w+").unwrap();
    let improved_input = unidecode(&input.to_lowercase());
    let input_words = regex
        .find_iter(&improved_input)
        .map(|s| s.as_str())
        .collect::<Vec<&str>>();
    let mut words_processed_to_tokens: Vec<&str> = Vec::default();
    let tokens: Vec<Token> = input_words
        .iter()
        .enumerate()
        .filter_map(|(pos, word)| {
            if pos % 2 == 0 && pos < input_words.len() - 1 {
                let normal_pair = word.to_string() + " " + input_words[pos + 1];
                let normal_pair_search_result =
                    find_most_similar_word_in_vocab(&normal_pair, &vocab, 90);

                if let Some((vocab_index, actual_word, weight)) = normal_pair_search_result {
                    words_processed_to_tokens.push(input_words[pos + 1]);
                    return Some(Token {
                        word: actual_word,
                        vocab_index,
                        weight,
                    });
                }
            }

            if !words_processed_to_tokens.contains(word) {
                let single_word_search_result = find_most_similar_word_in_vocab(word, &vocab, 90);

                if let Some((vocab_index, actual_word, weight)) = single_word_search_result {
                    return Some(Token {
                        word: actual_word,
                        vocab_index,
                        weight,
                    });
                }
            }

            None
        })
        .collect();
    let mut outputs = HashMap::with_capacity(categories.len());

    tokens.iter().enumerate().for_each(|(pos, token)| {
        let neighbours = get_token_neighbours(&tokens, pos)
            .iter()
            .map(|ng| ng.word.to_string())
            .collect::<Vec<String>>();
        let neighbours_occurrences: HashMap<(String, String), f32> =
            load_file(format!("weights/{}.bin", token.vocab_index), 0)
                .expect(&format!("could not find weights for the token {:?}", token));
        neighbours_occurrences
            .iter()
            .for_each(|((possible_neighbour, category), occurrences)| {
                if neighbours.contains(possible_neighbour) {
                    let old_count = outputs.get(category).unwrap_or(&0.0);
                    outputs.insert(category.to_string(), old_count + occurrences * token.weight);
                }
            });
    });

    outputs
}

const CATEGORIAS_TEXT: &str = include_str!("categorias.txt");

fn get_largest_output(outputs: &HashMap<String, f32>) -> Option<(String, f32)> {
    let keys = outputs.keys().collect::<Vec<&String>>();
    let largest = outputs
        .get(&keys.first()?.to_string())
        .expect("there is no category in the outputs");
    Some(outputs.iter().fold(
        (keys[0].to_string(), *largest),
        |(largest_cat, largest_occur), (cat, occur)| {
            if occur > &largest_occur {
                (cat.to_string(), *occur)
            } else {
                (largest_cat, largest_occur)
            }
        },
    ))
}

fn main() {
    let vocabulary: Vec<String> = load_file("vocab.bin", 0).expect("unable to load the vocabulary");
    let categories: Vec<&str> = CATEGORIAS_TEXT.split("\n").collect();

    let input_text = unidecode(
        &std::env::args()
            .skip(1)
            .fold(String::new(), |text, word| text + " " + &word)
            .trim()
            .to_lowercase(),
    );

    let outputs = predict(&input_text, &vocabulary, &categories);

    if let Some(output) = get_largest_output(&outputs) {
        println!("{:?}", output);
    }
}

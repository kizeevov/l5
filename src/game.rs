use itertools::Itertools;

use crate::{LetterModel, WordModel, WordsModel};

pub struct Game {
    pub words: WordsModel,
}

enum LetterState {
    None = 0,
    Missing = 1,
    Available = 2,
    InPlace = 3,
}

impl LetterModel {
    fn new() -> Self {
        Self {
            char: "".into(),
            state: LetterState::None as i32,
        }
    }
}

impl WordModel {
    fn new(number_letters: u8) -> Self {
        let letters = (0..number_letters)
            .into_iter()
            .map(|_| LetterModel::new())
            .collect_vec();

        Self {
            letters: slint::VecModel::from_slice(&letters),
        }
    }
}

impl WordsModel {
    fn new(number_word: u8, number_letters: u8) -> Self {
        let words: Vec<WordModel> = (0..number_word)
            .into_iter()
            .map(|_| WordModel::new(number_letters))
            .collect_vec();

        Self {
            current_word_index: 0,
            words: slint::VecModel::from_slice(&words),
        }
    }
}

impl Game {
    pub fn new_empty() -> Self {
        let words = WordsModel::new(5, 5);

        Self { words }
    }
}

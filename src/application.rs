use std::{
    collections::{HashMap, HashSet},
    io::{self, BufRead},
    rc::Rc,
};

use crate::{AppWindow, LetterModel, WordModel, WordSearcher};
use itertools::Itertools;
use slint::{ComponentHandle, Model};

pub struct AppState {
    main_window: slint::Weak<AppWindow>,
    words: Vec<WordModel>,
    nouns_cursor: io::Cursor<Vec<u8>>,
    letter_count: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LetterState {
    None = 0,
    Missing = 1,
    Available = 2,
    InPlace = 3,
}

impl LetterState {
    fn from_u32(val: i32) -> Self {
        match val {
            0 => LetterState::None,
            1 => LetterState::Missing,
            2 => LetterState::Available,
            3 => LetterState::InPlace,
            _ => todo!(),
        }
    }
}

impl AppState {
    pub fn new(
        letter_count: usize,
        main_window: slint::Weak<AppWindow>,
        nouns_bytes: &[u8],
    ) -> Rc<Self> {
        let nouns_cursor = io::Cursor::new(nouns_bytes.to_vec());
        let words: Vec<WordModel> = (0..5)
            .into_iter()
            .map(|_| WordModel::new(letter_count))
            .collect_vec();

        let app = Rc::new(Self {
            main_window,
            words,
            nouns_cursor,
            letter_count: 5,
        });

        let binding = app.main_window.unwrap();
        let word_searcher = binding.global::<WordSearcher>();

        word_searcher.set_words(app.words_model());

        let app_clone = Rc::clone(&app);
        word_searcher.on_key_pressed(move |key| app_clone.on_key_pressed(&key));

        let app_clone = Rc::clone(&app);
        word_searcher.on_letter_state_changed(move |row, letter_index, state| {
            app_clone.on_letter_state_changed(row as usize, letter_index as usize, state)
        });

        let app_clone = Rc::clone(&app);
        word_searcher.on_clear_all(move || app_clone.clear_all());

        let app_clone = Rc::clone(&app);
        word_searcher.on_random(move || app_clone.ranrom());

        app
    }

    pub fn run(&self) {
        self.main_window.unwrap().run()
    }

    fn words_model(&self) -> slint::ModelRc<WordModel> {
        slint::VecModel::from_slice(&self.words)
    }

    fn on_key_pressed(&self, key: &str) {
        let main_window = self.main_window.unwrap();
        let word_searcher = main_window.global::<WordSearcher>();
        let current_word_index = word_searcher.get_current_word_index() as usize;

        match key {
            "⟵" => self.remove_letter(current_word_index, word_searcher),
            "Enter" => self.check_words(),
            _ => self.add_letter(current_word_index, word_searcher, key),
        }
    }

    fn add_letter(&self, current_word_index: usize, word_searcher: WordSearcher, key: &str) {
        let word = match self.words.get(current_word_index) {
            Some(word) => word,
            None => return,
        };

        let letters = &word.letters;
        for index in 0..letters.row_count() {
            let mut letter = letters.row_data(index).unwrap();
            if letter.state == LetterState::None as i32 {
                letter.state = LetterState::Missing as i32;
                letter.char = key.into();
                letters.set_row_data(index, letter);
                break;
            }
        }

        let is_word_full = letters
            .iter()
            .all(|model| model.state != LetterState::None as i32);
        if is_word_full && (current_word_index < self.words.len() - 1) {
            word_searcher.set_current_word_index((current_word_index + 1) as i32)
        }
    }

    fn remove_letter(&self, current_word_index: usize, word_searcher: WordSearcher) {
        let word = match self.words.get(current_word_index) {
            Some(word) => word,
            None => return,
        };

        let letters = &word.letters;

        let has_letter = letters
            .iter()
            .any(|model| model.state != LetterState::None as i32);

        if !has_letter && current_word_index > 0 {
            word_searcher.set_current_word_index((current_word_index - 1) as i32);
            return;
        }

        for index in (0..letters.row_count()).rev() {
            let mut letter = letters.row_data(index).unwrap();
            if letter.state != LetterState::None as i32 {
                letter.clear();
                letters.set_row_data(index, letter);
                break;
            }
        }
    }

    fn on_letter_state_changed(&self, row: usize, letter_index: usize, state: i32) {
        let word = self.words.get(row).unwrap();
        let mut letter = word.letters.row_data(letter_index).unwrap();
        let mut state = state;

        if state > LetterState::InPlace as i32 {
            state = LetterState::Missing as i32;
        }

        letter.state = state as i32;
        word.letters.set_row_data(letter_index, letter);
    }

    fn check_words(&self) {
        let mut missing_letters = HashSet::new();
        let mut available_letters = HashMap::<char, HashSet<_>>::new();
        let mut in_place_letters = HashMap::new();

        for word in self.words.iter() {
            for (i, l) in word.letters.iter().enumerate() {
                let state = LetterState::from_u32(l.state);
                let char = l.char.to_lowercase().chars().next().unwrap_or_default();

                match state {
                    LetterState::None => continue,
                    LetterState::Missing => {
                        missing_letters.insert(char);
                    }
                    LetterState::Available => {
                        available_letters
                            .entry(char)
                            .and_modify(|indexes| {
                                indexes.insert(i);
                            })
                            .or_insert(HashSet::new());
                    }
                    LetterState::InPlace => {
                        in_place_letters.insert(char, i);
                    }
                };
            }
        }

        let lines = self.nouns_cursor.clone().lines().map(|l| l.unwrap());

        let search_words = lines
            .filter(|word| {
                let word = word.replace("ё", "е");
                let is_letters_count_correct = word.chars().count() == self.letter_count;
                let has_missing_letters = word.chars().any(|c| missing_letters.contains(&c));
                let has_available_letters = available_letters.iter().all(|(l, i)| {
                    match word.chars().position(|c| c == *l) {
                        Some(index) => !i.contains(&index),
                        None => false,
                    }
                });
                let has_in_place_letter = in_place_letters.iter().all(|(l, i)| {
                    match word.chars().position(|c| c == *l) {
                        Some(index) => index == *i || word.chars().filter(|cl| cl == l).count() > 1,
                        None => false,
                    }
                });

                is_letters_count_correct
                    && !has_missing_letters
                    && has_available_letters
                    && has_in_place_letter
            })
            .join("\n");

        self.update_search_words(&search_words);
    }

    fn update_search_words(&self, words: &str) {
        let main_window = self.main_window.unwrap();
        let word_searcher = main_window.global::<WordSearcher>();
        word_searcher.set_search_words(words.into())
    }

    fn clear_all(&self) {
        for word in self.words.iter() {
            let letters = &word.letters;
            for index in 0..letters.row_count() {
                let mut letter = letters.row_data(index).unwrap();
                letter.clear();
                letters.set_row_data(index, letter);
            }
        }

        self.update_search_words("");
    }

    fn ranrom(&self) {
        let lines = self.nouns_cursor.clone().lines().map(|l| l.unwrap());
        let words = lines
            .filter(|w| w.chars().count() == self.letter_count)
            .take(500)
            .join("\n");
        self.update_search_words(&words);
    }
}

impl LetterModel {
    fn new() -> Self {
        Self {
            char: "".into(),
            state: LetterState::None as i32,
        }
    }

    fn clear(&mut self) {
        self.char = "".into();
        self.state = LetterState::None as i32;
    }
}

impl WordModel {
    fn new(letter_count: usize) -> Self {
        let letters = (0..letter_count)
            .into_iter()
            .map(|_| LetterModel::new())
            .collect_vec();

        Self {
            letters: slint::VecModel::from_slice(&letters),
        }
    }
}

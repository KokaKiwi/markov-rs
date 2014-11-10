#![feature(phase, slicing_syntax)]
#![experimental]

/*!
 * Usage example:
 *
 * ```
 * use std::collections::HashMap;
 * use markov::MarkovGenerator;
 *
 * let mut markov = MarkovGenerator::new(HashMap::new());
 * markov.feed_from_words(&["Hello", "world", "my", "name", "is", "KokaKiwi"]);
 * let text = markov.generate_text(15);
 * ```
 */

#[phase(plugin, link)]
extern crate log;

use std::collections::HashMap;
use std::io::{
    BufferedReader,
    File,
};
use std::rand::{
    task_rng,
    Rng,
};

pub trait Cache {
    fn put(&mut self, key: (&str, &str), value: &str);
    fn get(&self, key: (&str, &str)) -> Option<&[String]>;

    fn has(&self, key: (&str, &str)) -> bool {
        self.get(key).is_some()
    }
}

impl Cache for HashMap<(String, String), Vec<String>> {
    fn put(&mut self, (w1, w2): (&str, &str), value: &str) {
        let w1 = w1.to_string();
        let w2 = w2.to_string();
        let value = value.to_string();

        if self.has((w1.as_slice(), w2.as_slice())) {
            self[(w1, w2)].push(value);
        } else {
            self.insert((w1, w2), vec![value]);
        }
    }

    fn get(&self, (w1, w2): (&str, &str)) -> Option<&[String]> {
        let w1 = w1.to_string();
        let w2 = w2.to_string();

        self.get(&(w1, w2)).map(|words| words.as_slice())
    }
}

pub struct MarkovGenerator<C: Cache> {
    pub cache: C,
    pub words: Vec<String>,
}

impl<C> MarkovGenerator<C> where C: Cache {
    pub fn new(cache: C) -> MarkovGenerator<C> {
        MarkovGenerator {
            cache: cache,
            words: Vec::new(),
        }
    }

    pub fn feed_from_words(&mut self, words: &[&str]) {
        {
            let last_words: Vec<&str> = if self.words.len() > 3 {
                self.words[self.words.len() - 3..]
                        .iter()
                        .map(|word| word.as_slice())
                        .collect()
            } else {
                Vec::new()
            };
            let mut triples = Triples::new(last_words.iter().chain(words.iter()));

            for (&w1, &w2, &w3) in triples {
                self.cache.put((w1, w2), w3);
            }
        }

        self.words.extend(words.iter().map(|s| s.to_string()));
    }

    pub fn feed_from_file(&mut self, path: &Path) {
        let mut reader = BufferedReader::new(File::open(path));

        for line in reader.lines() {
            let line = line.unwrap();

            let seps = |c: char| [' ', '\t', '\n', '\r'].contains(&c);
            let filter = |word: &str| !word.is_empty();
            let words: Vec<&str> = line
                                    .split(seps)
                                    .filter(|word| filter(*word))
                                    .collect();
            debug!("Collected words: {}", words);

            self.feed_from_words(words.as_slice());
        }
    }

    pub fn generate_text(&self, size: uint) -> String {
        let mut words: Vec<&str> = Vec::new();
        let mut rng = task_rng();

        let seed = rng.gen_range(0, self.words.len() - 3);
        let mut w1 = &self.words[seed];
        let mut w2 = &self.words[seed + 1];

        for _ in range(0, size) {
            words.push(w1.as_slice());

            let old_w1 = w1;
            w1 = w2;
            w2 = {
                let words = match self.cache.get((old_w1.as_slice(), w2.as_slice())) {
                    Some(words) => words,
                    None => break, // Break loop, we got no more words to put in the text.
                };
                rng.choose(words).unwrap()
            };
        }

        words.connect(" ")
    }
}

struct Triples<'a, T, I>
    where I: Iterator<&'a T> + Clone {
    iter: I,
}

impl<'a, T, I> Triples<'a T, I>
    where I: Iterator<&'a T> + Clone {
    pub fn new(iter: I) -> Triples<'a, T, I> {
        Triples {
            iter: iter,
        }
    }
}

impl<'a, T, I> Iterator<(&'a T, &'a T, &'a T)> for Triples<'a T, I>
    where I: Iterator<&'a T> + Clone {
    fn next(&mut self) -> Option<(&'a T, &'a T, &'a T)> {
        let a = self.iter.next();
        let mut iter = self.iter.clone();
        let b = iter.next();
        let c = iter.next();

        match (a, b, c) {
            (Some(a), Some(b), Some(c)) => Some((a, b, c)),
            _ => None,
        }
    }
}

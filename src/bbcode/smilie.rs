use std::cmp::Ordering;
use std::collections::HashMap;

pub struct Smilies(Vec<(String, String)>);

impl Default for Smilies {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl Smilies {
    pub fn new_from_hashmap(smilies: &HashMap<String, String>) -> Self {
        let smilies: Vec<(String, String)> = smilies
            .iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();

        Self(Self::sort_tuples(smilies))
    }

    pub fn new_from_tuples(smilies: Vec<(String, String)>) -> Self {
        Self(Self::sort_tuples(smilies))
    }

    fn sort_tuples(mut smilies: Vec<(String, String)>) -> Vec<(String, String)> {
        smilies.sort_by(|a, b| {
            if a.0.chars().count() > b.0.chars().count() {
                Ordering::Less
            } else if b.0.chars().count() > a.0.chars().count() {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
        smilies
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (String, String)> {
        self.0.iter()
    }
}

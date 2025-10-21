use std::collections::HashMap;

#[derive(Default)]
pub struct Smilies(Vec<(String, String)>);

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
        smilies.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
        //smilies.sort_by(|a, b| a.0.chars().cmp(b.0.chars()));
        smilies
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, (String, String)> {
        self.0.iter()
    }
}

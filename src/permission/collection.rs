use super::category::Category;
use super::error::Error;
use super::item::Item;
use super::GROUP_LIMIT;
use super::MAX_PERMS;
use dashmap::DashMap;

/// Organiztion struct.
/// Collection of Categories which each contain Items.
/// This represents all possible permissions.
#[derive(Clone, Debug)]
pub struct Collection {
    /// Group ID -> Category
    pub categories: [Category; GROUP_LIMIT as usize],
    /// Item Label -> Tuple (category index, permission index)
    pub dictionary: DashMap<String, (u8, u8)>,
    /// Item ID -> Tuple (category index, permission index)
    pub lookup: DashMap<i32, (u8, u8)>,
}

impl Default for Collection {
    fn default() -> Self {
        Collection {
            // TODO: This should be changed after release.
            // https://doc.rust-lang.org/nightly/core/array/fn.from_fn.html
            categories: [(); GROUP_LIMIT as usize].map(|_| Category::default()),
            //categories: [Category::default(); GROUP_LIMIT as usize],
            dictionary: DashMap::with_capacity(MAX_PERMS as usize),
            lookup: DashMap::with_capacity(MAX_PERMS as usize),
        }
    }
}

impl Collection {
    pub fn build_dictionary(&mut self) {
        let newd: DashMap<String, (u8, u8)> = DashMap::with_capacity(MAX_PERMS as usize);

        for (x, c) in self.categories.iter().enumerate() {
            for (y, i) in c.items.iter().enumerate() {
                if i.id > 0 {
                    newd.insert(i.label.to_owned(), (x as u8, y as u8));
                }
            }
        }

        self.dictionary = newd;
    }

    pub fn get_item_pos(&self, label: &str) -> Result<(usize, usize), Error> {
        match self.dictionary.get(label) {
            Some(tuple) => Ok((tuple.0 as usize, tuple.1 as usize)),
            None => Err(Error::PermissionNotFound),
        }
    }

    pub fn get_item(&self, label: &str) -> Result<&Item, Error> {
        match self.dictionary.get(label) {
            Some(tuple) => Ok(&self.categories[tuple.0 as usize].items[tuple.1 as usize]),
            None => Err(Error::PermissionNotFound),
        }
    }
}

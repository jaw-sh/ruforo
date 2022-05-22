use super::error::Error;
use super::item::Item;
use super::PERM_LIMIT;

/// Organization struct.
/// Permission category which may catalogue up to 64 permission items.
pub struct Category {
    pub id: i32,
    pub position: u8,
    pub items: [Item; PERM_LIMIT as usize],
}

impl Default for Category {
    /// Bafflingly, Default is manually implemented for generic types up to a length of 32
    /// See: https://doc.rust-lang.org/std/default/trait.Default.html#impl-Default-71
    fn default() -> Self {
        Category {
            id: 0,
            position: 0,
            // TODO: This should be changed after release.
            // https://doc.rust-lang.org/nightly/core/array/fn.from_fn.html
            items: [(); PERM_LIMIT as usize].map(|_| Item::default()),
            //items: [Item::default(); PERM_LIMIT as usize],
        }
    }
}

impl Category {
    pub fn add_item(&mut self, id: &i32, label: &str) -> Result<&mut Item, Error> {
        let mut i: u8 = 0;

        for item in self.items.iter_mut() {
            // Is a default permission
            if item.id == 0 {
                *item = Item {
                    id: *id,
                    category: self.id,
                    label: label.to_string(),
                    position: i,
                };
                return Ok(item);
            }

            i += 1;
        }

        Err(Error::CategoryOverflow)
    }

    /// Returns immutable Item reference by its database id.
    pub fn borrow_item_by_id(&self, id: i32) -> Option<&Item> {
        for item in self.items.iter() {
            if item.id == id {
                return Some(item);
            }
        }
        return None;
    }

    /// Returns immutable Item reference by its name.
    pub fn borrow_item_by_label(&self, label: String) -> Option<&Item> {
        for item in self.items.iter() {
            if item.label == label {
                return Some(item);
            }
        }
        return None;
    }

    /// Returns next available possible
    pub fn get_next_position(&self) -> Result<u8, Error> {
        let mut i: u8 = 0;
        for item in self.items.iter() {
            if item.id == 0 {
                return Ok(i);
            }
            i += 1;
        }
        return Err(Error::CategoryOverflow);
    }
}

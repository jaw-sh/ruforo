use super::PERM_LIMIT;
use super::item::Item;
use super::error::Error;

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
            items: [Item::default(); PERM_LIMIT as usize]
        }
    }
}

impl Category {
    pub fn add_item(&mut self, id: i32) -> Result<&mut Item, Error> {
        let mut i: u8 = 0;

        for item in self.items.iter_mut() {
            // Is a default permission
            if item.id == 0 {
                *item = Item { id, position: i };
                return Ok(item);
            }

            i += 1;
        }

        Err(Error::CategoryOverflow)
    }
}
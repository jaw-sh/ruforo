use super::collection_values::CollectionValues;
use super::GROUP_LIMIT;

/// Data struct containing all permission categories as final, evaluated masks.
pub struct Mask {
    pub categories: [u64; GROUP_LIMIT as usize],
}

impl Default for Mask {
    /// Default is manually implemented for generic types up to a length of 32
    /// See: https://doc.rust-lang.org/std/default/trait.Default.html#impl-Default-71
    fn default() -> Self {
        Self {
            categories: [0; GROUP_LIMIT as usize],
        }
    }
}

impl From<CollectionValues> for Mask {
    fn from(item: CollectionValues) -> Self {
        let mut mask = Self::default();
        for (i, category) in item.categories.iter().enumerate() {
            mask.categories[i] = u64::from(*category);
        }
        mask
    }
}

impl Mask {
    pub fn can(&self, category: usize, permission: i32) -> bool {
        self.categories[category] & (1 << permission) > 0
    }
}

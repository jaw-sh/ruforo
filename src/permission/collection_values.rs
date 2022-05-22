use super::category_values::CategoryValues;
use super::GROUP_LIMIT;

/// Data struct.
/// Collection of permission Values, organized by Category.
/// This represents all permissions set for a user or group on a resource.
#[derive(Clone)]
pub struct CollectionValues {
    /// Group ID -> CategoryValues
    pub categories: [CategoryValues; GROUP_LIMIT as usize],
}

impl Default for CollectionValues {
    /// Default is manually implemented for generic types up to a length of 32
    /// See: https://doc.rust-lang.org/std/default/trait.Default.html#impl-Default-71
    fn default() -> Self {
        CollectionValues {
            // TODO: This should be changed after release.
            // https://doc.rust-lang.org/nightly/core/array/fn.from_fn.html
            //categories: [(); GROUP_LIMIT as usize].map(|_| CategoryValues::default()),
            categories: [CategoryValues::default(); GROUP_LIMIT as usize],
        }
    }
}

impl CollectionValues {
    /// Combines permission sets at the same depth.
    /// Explicit YES permissions override explicit NO permissions.
    pub fn join(&self, left: &Self) -> Self {
        let mut i: usize = 0;
        let mut categories: [CategoryValues; GROUP_LIMIT as usize] = Default::default();

        for values in categories.iter_mut() {
            *values = self.categories[i].join(&left.categories[i]);
            i += 1;
        }

        Self { categories }
    }

    /// Combines permission sets vertically.
    /// No permissions override Yes permissions.
    pub fn stack(&self, below: &Self) -> Self {
        let mut i: usize = 0;
        let mut categories: [CategoryValues; GROUP_LIMIT as usize] = Default::default();

        for values in categories.iter_mut() {
            *values = self.categories[i].stack(&below.categories[i]);
            i += 1;
        }

        Self { categories }
    }
}
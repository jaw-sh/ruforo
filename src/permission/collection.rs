use super::GROUP_LIMIT;
use super::category::Category;

/// Organiztion struct.
/// Collection of Categories which each contain Items.
/// This represents all possible permissions.
#[derive(Default)]
pub struct Collection {
    /// Group ID -> CategoryValues
    pub categories: [Category; GROUP_LIMIT as usize]
}
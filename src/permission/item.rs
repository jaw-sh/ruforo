/// Organization struct.
/// Identifying information for a single permission option.
#[derive(Clone, Default, Debug)]
pub struct Item {
    /// Database ID for this Permission Item.
    pub id: i32,
    /// Database ID for the Category this Item sits in.
    pub category: i32,
    /// Name string for the permission.
    pub label: String,
    /// 0 indexed position of this item in the Category.
    pub position: u8,
}

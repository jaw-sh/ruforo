/// Organization struct.
/// Identifying information for a single permission option.
#[derive(Clone, Default, Debug)]
pub struct Item {
    pub id: i32,
    pub category: i32,
    pub label: String,
    pub position: u8,
}

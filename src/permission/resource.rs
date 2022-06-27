/// Organiztion struct.
/// Relational data used for finalizing permission masks in real use.
pub struct Resource {
    pub id: i32,
    pub parent: Option<i32>,
    pub children: Option<Vec<i32>>,
}

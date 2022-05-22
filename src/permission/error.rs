/// Permission data and mask errors.
pub enum Error {
    /// Category has reached GROUP_LIMIT and cannot add more Item.
    CategoryOverflow,
    /// Requested permission does not exist in our collection.
    PermissionNotFound,
}

/// Value set for a single permission.
pub enum Flag {
    /// Grants permission
    YES = 1,
    /// Unset permission
    DEFAULT = 0,
    /// Removes any existing YES permission
    NO = -1,
    /// Never permitted, cannot be re-permitted
    NEVER = -2,
}

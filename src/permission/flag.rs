use sea_orm::entity::prelude::{DeriveActiveEnum, EnumIter};

/// Value set for a single permission.
/// Compatible with sea_orm enum type.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum Flag {
    /// Grants permission
    #[sea_orm(num_value = 1)]
    YES = 1,
    /// Unset permission
    #[sea_orm(num_value = 0)]
    DEFAULT = 0,
    /// Removes any existing YES permission
    #[sea_orm(num_value = -1)]
    NO = -1,
    /// Never permitted, cannot be re-permitted
    #[sea_orm(num_value = -2)]
    NEVER = -2,
}

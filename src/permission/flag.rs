use sea_orm::entity::prelude::{DeriveActiveEnum, EnumIter};

/// Value set for a single permission.
/// Compatible with sea_orm enum type.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "password_cipher")]
pub enum Flag {
    /// Grants permission
    #[sea_orm(string_value = "yes")]
    YES = 1,
    /// Unset permission
    #[sea_orm(string_value = "default")]
    DEFAULT = 0,
    /// Removes any existing YES permission
    #[sea_orm(string_value = "no")]
    NO = -1,
    /// Never permitted, cannot be re-permitted
    #[sea_orm(string_value = "never")]
    NEVER = -2,
}

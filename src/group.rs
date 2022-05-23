use sea_orm::entity::prelude::{DeriveActiveEnum, EnumIter};

/// Value set for a single permission.
/// Compatible with sea_orm enum type.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "password_cipher")]
pub enum GroupType {
    /// Not a system group (may be deleted).
    #[sea_orm(string_value = "normal")]
    NORMAL,
    /// System group for any anonymous connection (i.e. Tor)
    #[sea_orm(string_value = "system_anon")]
    SYSTEM_ANON,
    /// System group for guests and unconfirmed accounts.
    #[sea_orm(string_value = "system_guest")]
    SYSTEM_GUEST,
    /// System group for signed-in, confirmed users.
    #[sea_orm(string_value = "system_user")]
    SYSTEM_USER,
}

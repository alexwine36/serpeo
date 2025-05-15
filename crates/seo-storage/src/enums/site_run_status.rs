use sea_orm::entity::prelude::*;

#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    EnumIter,
    DeriveActiveEnum,
    serde::Serialize,
    serde::Deserialize,
    specta::Type,
)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
pub enum SiteRunStatus {
    #[sea_orm(string_value = "Pending")]
    Pending,
    #[sea_orm(string_value = "Running")]
    Running,
    #[sea_orm(string_value = "Finished")]
    Finished,
    #[sea_orm(string_value = "Error")]
    Error,
}

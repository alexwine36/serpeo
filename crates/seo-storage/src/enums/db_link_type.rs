use sea_orm::entity::prelude::*;
use seo_plugins::utils::link_parser::LinkType;

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
pub enum DbLinkType {
    #[sea_orm(string_value = "Internal")]
    Internal,
    #[sea_orm(string_value = "External")]
    External,
    #[sea_orm(string_value = "Mailto")]
    Mailto,
    #[sea_orm(string_value = "Tel")]
    Tel,
    #[sea_orm(string_value = "Unknown")]
    Unknown,
}

impl From<LinkType> for DbLinkType {
    fn from(link_type: LinkType) -> Self {
        match link_type {
            LinkType::Internal => DbLinkType::Internal,
            LinkType::External => DbLinkType::External,
            LinkType::Mailto => DbLinkType::Mailto,
            LinkType::Tel => DbLinkType::Tel,
            LinkType::Unknown => DbLinkType::Unknown,
        }
    }
}

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

// const RULE_ID_FK_NAME: &str = "fk_page_rule_result_plugin_rule";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let create_table = Table::create()
            .table(PluginRule::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(PluginRule::Id)
                    .string()
                    .not_null()
                    .primary_key(),
            )
            .col(string(PluginRule::Name))
            .col(string(PluginRule::PluginName))
            .col(string(PluginRule::Description))
            .col(string(PluginRule::Severity))
            .col(string(PluginRule::Category))
            .col(string(PluginRule::RuleType))
            .col(string(PluginRule::PassedMessage))
            .col(string(PluginRule::FailedMessage))
            .col(boolean(PluginRule::Enabled).default(true))
            .to_owned();
        let create_table = timestamps(create_table);

        manager.create_table(create_table.to_owned()).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(PluginRule::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum PluginRule {
    Table,
    Id,
    Name,
    PluginName,
    Description,
    Severity,
    Category,
    RuleType,
    PassedMessage,
    FailedMessage,
    Enabled,
}

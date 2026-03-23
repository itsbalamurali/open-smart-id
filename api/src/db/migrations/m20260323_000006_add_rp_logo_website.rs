use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RelyingParties::Table)
                    .add_column(ColumnDef::new(RelyingParties::LogoUrl).text().null())
                    .add_column(ColumnDef::new(RelyingParties::WebsiteUrl).text().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(RelyingParties::Table)
                    .drop_column(RelyingParties::LogoUrl)
                    .drop_column(RelyingParties::WebsiteUrl)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum RelyingParties {
    Table,
    LogoUrl,
    WebsiteUrl,
}

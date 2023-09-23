use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(LoginEntry::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(LoginEntry::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(LoginEntry::Username).string().not_null())
                    .col(
                        ColumnDef::new(LoginEntry::Timestamp)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .col(ColumnDef::new(LoginEntry::Success).boolean().not_null())
                    .col(ColumnDef::new(LoginEntry::IpAddress).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(BasicLoginUser::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BasicLoginUser::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BasicLoginUser::Username)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(BasicLoginUser::Password).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AccessEntry::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AccessEntry::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AccessEntry::Username).string().not_null())
                    .col(
                        ColumnDef::new(AccessEntry::Timestamp)
                            .date_time()
                            .not_null()
                            .default(SimpleExpr::Keyword(Keyword::CurrentTimestamp)),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(LoginEntry::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(BasicLoginUser::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(AccessEntry::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum AccessEntry {
    Table,
    Id,
    Timestamp,
    Username,
}

#[derive(DeriveIden)]
enum LoginEntry {
    Table,
    Id,
    Timestamp,
    Username,
    Success,
    IpAddress,
}

#[derive(DeriveIden)]
enum BasicLoginUser {
    Table,
    Id,
    Username,
    Password,
}

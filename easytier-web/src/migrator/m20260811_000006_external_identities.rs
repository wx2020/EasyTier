use sea_orm_migration::{prelude::*, schema::*};

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20260811_000006_external_identities"
    }
}

#[derive(DeriveIden)]
enum ExternalIdentities {
    Table,
    Id,
    Issuer,
    Subject,
    UserId,
    CreateTime,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ExternalIdentities::Table)
                    .if_not_exists()
                    .col(pk_auto(ExternalIdentities::Id))
                    .col(text(ExternalIdentities::Issuer).not_null())
                    .col(text(ExternalIdentities::Subject).not_null())
                    .col(integer(ExternalIdentities::UserId).not_null())
                    .col(timestamp_with_time_zone(ExternalIdentities::CreateTime).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_external_identities_user_id")
                            .from(ExternalIdentities::Table, ExternalIdentities::UserId)
                            .to(Users::Table, Users::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_external_identities_issuer_subject")
                    .table(ExternalIdentities::Table)
                    .col(ExternalIdentities::Issuer)
                    .col(ExternalIdentities::Subject)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_external_identities_user_id")
                    .table(ExternalIdentities::Table)
                    .col(ExternalIdentities::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ExternalIdentities::Table).to_owned())
            .await
    }
}

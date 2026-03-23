use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Sessions::Table)
                    .if_not_exists()
                    .col(string(Sessions::Id).primary_key())
                    .col(string(Sessions::RelyingPartyId))
                    .col(string_null(Sessions::AccountId))
                    // Session type & state
                    .col(string(Sessions::Kind))
                    .col(string_len(Sessions::State, 20).default("RUNNING"))
                    // Device link fields
                    .col(string_null(Sessions::SessionToken))
                    .col(string_null(Sessions::SessionSecret))
                    .col(string_null(Sessions::DeviceLinkBase))
                    // Request parameters
                    .col(string_null(Sessions::SignatureProtocol))
                    .col(string_null(Sessions::SignatureAlgorithm))
                    .col(string_null(Sessions::HashAlgorithm))
                    .col(string_null(Sessions::CertificateLevel))
                    .col(text_null(Sessions::ChallengeOrDigest))
                    .col(text_null(Sessions::Interactions))
                    .col(string_null(Sessions::Nonce))
                    .col(string_null(Sessions::InitialCallbackUrl))
                    .col(string_null(Sessions::LinkedSessionId))
                    .col(string_null(Sessions::FlowType))
                    // Verification code
                    .col(string_null(Sessions::VcType))
                    .col(string_null(Sessions::VcValue))
                    // Request properties
                    .col(boolean(Sessions::ShareMdClientIpAddress).default(false))
                    // Result fields
                    .col(string_null(Sessions::EndResult))
                    .col(string_null(Sessions::DocumentNumber))
                    .col(text_null(Sessions::SignatureJson))
                    .col(string_null(Sessions::ServerRandom))
                    .col(string_null(Sessions::UserChallenge))
                    .col(text_null(Sessions::CertValue))
                    .col(string_null(Sessions::CertLevel))
                    .col(string_null(Sessions::InteractionTypeUsed))
                    .col(string_null(Sessions::DeviceIpAddress))
                    .col(text_null(Sessions::IgnoredPropertiesJson))
                    // Timestamps
                    .col(timestamp(Sessions::CreatedAt))
                    .col(timestamp(Sessions::UpdatedAt))
                    // Foreign keys
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sessions::Table, Sessions::RelyingPartyId)
                            .to(RelyingParties::Table, RelyingParties::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Sessions::Table, Sessions::AccountId)
                            .to(Accounts::Table, Accounts::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for common lookups
        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_relying_party_id")
                    .col(Sessions::RelyingPartyId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_account_id")
                    .col(Sessions::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Sessions::Table)
                    .name("idx_sessions_state")
                    .col(Sessions::State)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Sessions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Sessions {
    Table,
    Id,
    RelyingPartyId,
    AccountId,
    Kind,
    State,
    SessionToken,
    SessionSecret,
    DeviceLinkBase,
    SignatureProtocol,
    SignatureAlgorithm,
    HashAlgorithm,
    CertificateLevel,
    ChallengeOrDigest,
    Interactions,
    Nonce,
    InitialCallbackUrl,
    LinkedSessionId,
    FlowType,
    VcType,
    VcValue,
    ShareMdClientIpAddress,
    EndResult,
    DocumentNumber,
    SignatureJson,
    ServerRandom,
    UserChallenge,
    CertValue,
    CertLevel,
    InteractionTypeUsed,
    DeviceIpAddress,
    IgnoredPropertiesJson,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RelyingParties {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Accounts {
    Table,
    Id,
}

use sea_orm_migration::prelude::*;

mod m20260323_000001_create_relying_parties;
mod m20260323_000002_create_accounts;
mod m20260323_000003_create_certificates;
mod m20260323_000004_create_sessions;
mod m20260323_000005_create_devices;
mod m20260323_000006_add_rp_logo_website;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260323_000001_create_relying_parties::Migration),
            Box::new(m20260323_000002_create_accounts::Migration),
            Box::new(m20260323_000003_create_certificates::Migration),
            Box::new(m20260323_000004_create_sessions::Migration),
            Box::new(m20260323_000005_create_devices::Migration),
            Box::new(m20260323_000006_add_rp_logo_website::Migration),
        ]
    }
}

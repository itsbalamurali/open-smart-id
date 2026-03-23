use sea_orm_migration::prelude::*;

mod m20260323_000001_create_relying_parties;
mod m20260323_000002_create_accounts;
mod m20260323_000003_create_certificates;
mod m20260323_000004_create_sessions;
mod m20260323_000005_create_devices;
mod m20260323_000006_add_rp_logo_website;
mod m20260323_000007_create_admin_users;
mod m20260323_000008_create_roles;
mod m20260323_000009_create_admin_user_roles;
mod m20260323_000010_create_audit_logs;
mod m20260323_000011_seed_rbac;

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
            Box::new(m20260323_000007_create_admin_users::Migration),
            Box::new(m20260323_000008_create_roles::Migration),
            Box::new(m20260323_000009_create_admin_user_roles::Migration),
            Box::new(m20260323_000010_create_audit_logs::Migration),
            Box::new(m20260323_000011_seed_rbac::Migration),
        ]
    }
}

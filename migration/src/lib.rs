pub use sea_orm_migration::prelude::*;

mod m20230723_215117_create_users_table;
mod m20230723_215203_create_files_table;
mod m20230803_151846_create_auth_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20230723_215117_create_users_table::Migration),
            Box::new(m20230723_215203_create_files_table::Migration),
            Box::new(m20230803_151846_create_auth_table::Migration),
        ]
    }
}

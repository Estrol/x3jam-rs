use sea_orm_migration::{MigrationTrait, MigratorTrait};

pub mod equipment;
pub mod item;
pub mod user;
pub mod session;
pub mod score;

pub struct Migrator;

impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(user::Migration),
            Box::new(item::Migration),
            Box::new(equipment::Migration),
            Box::new(session::Migration),
            Box::new(score::Migration),
        ]
    }
}

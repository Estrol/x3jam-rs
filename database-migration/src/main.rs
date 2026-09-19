use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

pub mod commandline;
pub mod config;
pub mod migrations;

#[tokio::main]
async fn main() {
    commandline::init();
    config::init();

    let url: Option<String> = {
        let cli = commandline::get_str("--url", "");
        if !cli.is_empty() {
            Some(cli)
        } else {
            let cfg = config::get_str("DATABASE", "URL", "");
            if !cfg.is_empty() { Some(cfg) } else { None }
        }
    };

    if url.is_none() {
        eprintln!("Database URL not provided. Please provide it via command line or config.");
        std::process::exit(1);
    }

    let db_connection = Database::connect(url.unwrap())
        .await
        .expect("Failed to connect to the database");

    let is_up = commandline::contains("up");
    let is_down = commandline::contains("down");

    if is_up && is_down {
        eprintln!("Cannot specify both --up and --down. Please choose one.");
        std::process::exit(1);
    }

    // default is_up if neither is specified
    if !is_up && !is_down {
        migrations::Migrator::up(&db_connection, None)
            .await
            .expect("Failed to run migrations");

        println!("Migrations applied successfully.");
    } else if is_up {
        migrations::Migrator::up(&db_connection, None)
            .await
            .expect("Failed to run migrations");

        println!("Migrations applied successfully.");
    } else if is_down {
        migrations::Migrator::down(&db_connection, None)
            .await
            .expect("Failed to revert migrations");

        println!("Migrations reverted successfully.");
    }
}

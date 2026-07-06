use std::sync::OnceLock;

static DATABASE: OnceLock<database::GameDatabase> = OnceLock::new();

pub async fn init() {
    let connection_string = super::config::get_str("DATABASE", "URL", "");
    if connection_string.is_empty() {
        panic!("DATABASE URL is not set in the configuration");
    }

    let migrate = super::commandline::contains("migrate");

    // TODO: handle errors properly instead of panicking
    let db = database::GameDatabase::new(&connection_string, migrate).await;

    #[cfg(debug_assertions)]
    {
        if db.get_user_by_name("testuser").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser",
                &hash,
                "TestUser",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser2").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser2",
                &hash,
                "TestUser2",
                database::CharacterGender::Female,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser3").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser3",
                &hash,
                "TestUser3",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser4").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser4",
                &hash,
                "TestUser4",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser5").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser5",
                &hash,
                "TestUser5",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser6").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser6",
                &hash,
                "TestUser6",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser7").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser7",
                &hash,
                "TestUser7",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }

        if db.get_user_by_name("testuser8").await.is_none() {
            let hash =
                bcrypt::hash("password", bcrypt::DEFAULT_COST).expect("Failed to hash password");

            db.create_user(
                "testuser8",
                &hash,
                "TestUser8",
                database::CharacterGender::Male,
            )
            .await
            .expect("Failed to create debug user");
        }
    }

    DATABASE.set(db).expect("Failed to set database");
}

pub fn get() -> &'static database::GameDatabase {
    DATABASE.get().expect("Database not initialized")
}

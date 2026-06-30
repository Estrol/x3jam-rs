use sea_orm::{
    ActiveValue::{NotSet, Set},
    Database, QueryOrder as _, QuerySelect as _, TransactionTrait as _,
    entity::prelude::*,
};
use sea_orm_migration::prelude::*;

pub mod models;

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub password_hash: String,
    pub nickname: String,
    pub exp: u64,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
    pub mcash: u32,
    pub point: u32,
    pub o2gems: u32,
    pub gender: CharacterGender,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub slot: u8,
    pub item_id: u32,
    pub quantity: u32,
}

#[derive(Debug, Copy, Default, Clone, encoder::StructSerializer, encoder::StructDeserializer)]
pub struct Equipment {
    pub instrument: u32,
    pub hair: u32,
    pub accessory: u32,
    pub glove: u32,
    pub necklace: u32,
    pub cloth: u32,
    pub pant: u32,
    pub glass: u32,
    pub earring: u32,
    pub cloth_accessory: u32,
    pub shoes: u32,
    pub face: u32,
    pub wing: u32,
    pub hair_accessory: u32,
    pub pet: u32,
    pub instrument_accessory: u32,
}

#[derive(Debug, Clone)]
pub struct Score {
    pub id: u32,
    pub user_id: u64,
    pub music_id: u32,
    pub score: u32,
    pub cool: u32,
    pub good: u32,
    pub bad: u32,
    pub miss: u32,
    pub max_combo: u32,
    pub jam_combo: u32,
    pub timing: u32,
    pub rate: f32,
    pub fln: u32,
    pub sln: u32,
    pub nln: u32,
    pub arragement: [u8; 7],
    pub skills: Vec<u32>,
    pub timestamp: DateTimeUtc,
}

const FEMALE_DEFAULT_EQUIPMENT: [u32; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 0, 0, 0, 0, 0];
const MALE_DEFAULT_EQUIPMENT: [u32; 16] = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 35, 0, 0, 0, 0, 0];

#[repr(u8)]
#[derive(Debug, Clone)]
pub enum CharacterGender {
    Male = 0,
    Female = 1,
}

#[derive(Debug, Clone)]
pub struct UserAuthenticationInfo {
    pub id: u64,
    pub password_hash: String,
}

#[derive(Debug)]
pub struct GameDatabase {
    pub connection: DatabaseConnection,
    pub default_equipment_male: [u32; 16],
    pub default_equipment_female: [u32; 16],
}

pub fn replace_if_needed(connection: &mut String) {
    const NEED_REPLACE_PREFIXES: [[&str; 2]; 1] = [
        // mariadb not supported in sqlx but compatible with mysql driver,
        // so we replace the prefix to allow using mariadb connection strings
        ["mariadb://", "mysql://"],
    ];

    for [old_prefix, new_prefix] in NEED_REPLACE_PREFIXES.iter() {
        if connection.starts_with(old_prefix) {
            *connection = connection.replacen(old_prefix, new_prefix, 1);
            break;
        }
    }
}

pub fn is_supported_driver(connection: &mut String) -> bool {
    const SUPPORTED_DRIVERS: [&str; 2] = ["mysql", "mariadb"];

    for driver in SUPPORTED_DRIVERS.iter() {
        if connection.starts_with(driver) {
            replace_if_needed(connection);

            return true;
        }
    }

    false
}

impl GameDatabase {
    pub async fn new(connection: &str) -> Self {
        let mut connection = connection.to_string();
        if !is_supported_driver(&mut connection) {
            panic!("Unsupported database driver. Supported drivers: mysql, mariadb");
        }

        let db_connection = Database::connect(connection)
            .await
            .expect("Failed to connect to the database");

        models::migrations::Migrator::up(&db_connection, None)
            .await
            .expect("Failed to run migrations");

        models::session::Entity::delete_many()
            .exec(&db_connection)
            .await
            .expect("Failed to clear sessions table on startup");

        GameDatabase {
            connection: db_connection,
            default_equipment_female: FEMALE_DEFAULT_EQUIPMENT,
            default_equipment_male: MALE_DEFAULT_EQUIPMENT,
        }
    }

    // Convenience method for creating a GameDatabase instance from MySQL/MariaDB connection parameters
    pub async fn from_mysql(address: &str, username: &str, password: &str, database: &str) -> Self {
        let connection_uri = format!("mysql://{}:{}@{}/{}", username, password, address, database);
        Self::new(&connection_uri).await
    }

    pub fn set_default_equipment(&mut self, gender: CharacterGender, data: [u32; 16]) {
        match gender {
            CharacterGender::Male => self.default_equipment_male = data,
            CharacterGender::Female => self.default_equipment_female = data,
        };
    }

    pub async fn get_user_authentication_info(
        &self,
        username: &str,
    ) -> Option<UserAuthenticationInfo> {
        models::user::Entity::find()
            .filter(models::user::Column::Name.eq(username))
            .one(&self.connection)
            .await
            .expect("Failed to query user authentication info")
            .map(|model| UserAuthenticationInfo {
                id: model.id,
                password_hash: model.password_hash,
            })
    }

    pub async fn get_user_by_id(&self, user_id: u64) -> Option<UserInfo> {
        models::user::Entity::find_by_id(user_id)
            .one(&self.connection)
            .await
            .expect("Failed to query user by ID")
            .map(|model| UserInfo {
                id: model.id,
                name: model.name,
                password_hash: model.password_hash,
                nickname: model.nickname,
                exp: model.exp,
                wins: model.wins,
                losses: model.losses,
                draws: model.draws,
                mcash: model.mcash,
                point: model.point,
                o2gems: model.o2gems,
                gender: match model.gender {
                    models::user::Gender::Male => CharacterGender::Male,
                    models::user::Gender::Female => CharacterGender::Female,
                },
            })
    }

    pub async fn get_user_by_name(&self, username: &str) -> Option<UserInfo> {
        models::user::Entity::find()
            .filter(models::user::Column::Name.eq(username))
            .one(&self.connection)
            .await
            .expect("Failed to query user by name")
            .map(|model| UserInfo {
                id: model.id,
                name: model.name,
                password_hash: model.password_hash,
                nickname: model.nickname,
                exp: model.exp,
                wins: model.wins,
                losses: model.losses,
                draws: model.draws,
                mcash: model.mcash,
                point: model.point,
                o2gems: model.o2gems,
                gender: match model.gender {
                    models::user::Gender::Male => CharacterGender::Male,
                    models::user::Gender::Female => CharacterGender::Female,
                },
            })
    }

    pub async fn create_user(
        &self,
        username: &str,
        password_hash: &str,
        nickname: &str,
        gender: CharacterGender,
    ) -> Option<UserInfo> {
        let new_user = models::user::ActiveModel {
            name: Set(username.to_string()),
            password_hash: Set(password_hash.to_string()),
            nickname: Set(nickname.to_string()),
            gender: Set(match gender {
                CharacterGender::Female => models::user::Gender::Female,
                CharacterGender::Male => models::user::Gender::Male,
            }),
            ..Default::default()
        };

        let insert_result = models::user::Entity::insert(new_user)
            .exec(&self.connection)
            .await
            .expect("Failed to insert new user");

        let default_equipment = match gender {
            CharacterGender::Female => self.default_equipment_female,
            CharacterGender::Male => self.default_equipment_male,
        };

        let equipment = models::equipment::ActiveModel {
            id: NotSet,
            user_id: Set(insert_result.last_insert_id),
            instrument: Set(default_equipment[0]),
            hair: Set(default_equipment[1]),
            accessory: Set(default_equipment[2]),
            glove: Set(default_equipment[3]),
            necklace: Set(default_equipment[4]),
            cloth: Set(default_equipment[5]),
            pant: Set(default_equipment[6]),
            glass: Set(default_equipment[7]),
            earring: Set(default_equipment[8]),
            shoes: Set(default_equipment[9]),
            face: Set(default_equipment[10]),
            wing: Set(default_equipment[11]),
            hair_accessory: Set(default_equipment[12]),
            instrument_accessory: Set(default_equipment[13]),
            cloth_accessory: Set(default_equipment[14]),
            pet: Set(default_equipment[15]),
        };

        models::equipment::Entity::insert(equipment)
            .exec(&self.connection)
            .await
            .expect("Failed to insert equipment for new user");

        self.get_user_by_id(insert_result.last_insert_id).await
    }

    pub async fn save_user(
        &self,
        users: &[&UserInfo],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if users.is_empty() {
            return Ok(());
        }

        if users.len() > 1 {
            let txn = self.connection.begin().await?;

            let updated_users = users
                .iter()
                .map(|user| models::user::ActiveModel {
                    id: Set(user.id),
                    name: Set(user.name.clone()),
                    password_hash: Set(user.password_hash.clone()),
                    nickname: Set(user.nickname.clone()),
                    exp: Set(user.exp),
                    wins: Set(user.wins),
                    losses: Set(user.losses),
                    draws: Set(user.draws),
                    mcash: Set(user.mcash),
                    point: Set(user.point),
                    o2gems: Set(user.o2gems),
                    gender: Set(match user.gender {
                        CharacterGender::Female => models::user::Gender::Female,
                        CharacterGender::Male => models::user::Gender::Male,
                    }),
                })
                .collect::<Vec<_>>();

            for updated_user in updated_users {
                models::user::Entity::update(updated_user)
                    .exec(&txn)
                    .await
                    .expect("Failed to update user");
            }

            txn.commit().await?;
        } else {
            let user = &users[0];

            let update = models::user::ActiveModel {
                id: Set(user.id),
                name: Set(user.name.clone()),
                password_hash: Set(user.password_hash.clone()),
                nickname: Set(user.nickname.clone()),
                exp: Set(user.exp),
                wins: Set(user.wins),
                losses: Set(user.losses),
                draws: Set(user.draws),
                mcash: Set(user.mcash),
                point: Set(user.point),
                o2gems: Set(user.o2gems),
                gender: Set(match user.gender {
                    CharacterGender::Female => models::user::Gender::Female,
                    CharacterGender::Male => models::user::Gender::Male,
                }),
            };

            models::user::Entity::update(update)
                .exec(&self.connection)
                .await
                .expect("Failed to update user");
        }

        Ok(())
    }

    pub async fn delete_user(&self, user_id: u32) -> bool {
        let delete_result = models::user::Entity::delete_by_id(user_id)
            .exec(&self.connection)
            .await
            .expect("Failed to delete user");

        delete_result.rows_affected > 0
    }

    pub async fn get_inventory(&self, user_id: u64) -> Vec<Item> {
        models::item::Entity::find()
            .filter(models::item::Column::UserId.eq(user_id))
            .limit(30)
            .all(&self.connection)
            .await
            .expect("Failed to query inventory")
            .into_iter()
            .map(|model| Item {
                slot: model.slot,
                item_id: model.item_id,
                quantity: model.quantity,
            })
            .collect()
    }

    pub async fn save_inventory(
        &self,
        user_id: u64,
        inventory: &[Item],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let txn = self.connection.begin().await?;

        models::item::Entity::delete_many()
            .filter(models::item::Column::UserId.eq(user_id))
            .exec(&txn)
            .await
            .expect("Failed to delete existing inventory");

        if !inventory.is_empty() {
            let active_models: Vec<models::item::ActiveModel> = inventory
                .iter()
                .map(|item| models::item::ActiveModel {
                    id: NotSet,
                    user_id: Set(user_id),
                    slot: Set(item.slot),
                    item_id: Set(item.item_id),
                    quantity: Set(item.quantity),
                })
                .collect();

            models::item::Entity::insert_many(active_models)
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;

        Ok(())
    }

    pub async fn get_equipment(&self, user_id: u32) -> Option<Equipment> {
        models::equipment::Entity::find_by_id(user_id)
            .one(&self.connection)
            .await
            .expect("Failed to query equipment")
            .map(|model| Equipment {
                instrument: model.instrument,
                hair: model.hair,
                accessory: model.accessory,
                glove: model.glove,
                necklace: model.necklace,
                cloth: model.cloth,
                pant: model.pant,
                glass: model.glass,
                earring: model.earring,
                cloth_accessory: model.cloth_accessory,
                shoes: model.shoes,
                face: model.face,
                wing: model.wing,
                hair_accessory: model.hair_accessory,
                instrument_accessory: model.instrument_accessory,
                pet: model.pet,
            })
    }

    pub async fn save_equipment(
        &self,
        user_id: u64,
        equipment: &Equipment,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let updated_equipment = models::equipment::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
            instrument: Set(equipment.instrument),
            hair: Set(equipment.hair),
            accessory: Set(equipment.accessory),
            glove: Set(equipment.glove),
            necklace: Set(equipment.necklace),
            cloth: Set(equipment.cloth),
            pant: Set(equipment.pant),
            glass: Set(equipment.glass),
            earring: Set(equipment.earring),
            shoes: Set(equipment.shoes),
            face: Set(equipment.face),
            wing: Set(equipment.wing),
            hair_accessory: Set(equipment.hair_accessory),
            instrument_accessory: Set(equipment.instrument_accessory),
            cloth_accessory: Set(equipment.cloth_accessory),
            pet: Set(equipment.pet),
        };

        models::equipment::Entity::update_many()
            .set(updated_equipment)
            .filter(models::equipment::Column::UserId.eq(user_id as u32))
            .exec(&self.connection)
            .await
            .expect("Failed to update equipment");

        Ok(())
    }

    pub async fn create_session(
        &self,
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let existing_session = models::session::Entity::find()
            .filter(models::session::Column::UserId.eq(user_id))
            .one(&self.connection)
            .await?;

        if existing_session.is_some() {
            return Ok(false);
        }

        let new_session = models::session::ActiveModel {
            id: NotSet,
            user_id: Set(user_id),
        };

        models::session::Entity::insert(new_session)
            .exec(&self.connection)
            .await?;

        Ok(true)
    }

    pub async fn delete_session(
        &self,
        user_id: u64,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let delete_result = models::session::Entity::delete_many()
            .filter(models::session::Column::UserId.eq(user_id))
            .exec(&self.connection)
            .await?;

        Ok(delete_result.rows_affected > 0)
    }

    pub async fn submit_scores(
        &self,
        scores: &[Score],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if scores.is_empty() {
            return Ok(());
        }

        let active_models: Vec<models::score::ActiveModel> = scores
            .iter()
            .map(|score| models::score::ActiveModel {
                id: NotSet,
                user_id: Set(score.user_id),
                music_id: Set(score.music_id),
                score: Set(score.score),
                cool: Set(score.cool as u16),
                good: Set(score.good as u16),
                bad: Set(score.bad as u16),
                miss: Set(score.miss as u16),
                max_combo: Set(score.max_combo as u16),
                jam_combo: Set(score.jam_combo as u16),
                timing: Set(score.timing),
                rate: Set(score.rate),
                fln: Set(score.fln),
                sln: Set(score.sln),
                nln: Set(score.nln),
                arragement: Set(score
                    .arragement
                    .iter()
                    .map(|&n| n.to_string())
                    .collect::<Vec<String>>()
                    .join("")),
                skills: Set(score
                    .skills
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(",")),
                timestamp: Set(score.timestamp),
            })
            .collect();

        models::score::Entity::insert_many(active_models)
            .on_conflict(
                OnConflict::columns([
                    models::score::Column::UserId,
                    models::score::Column::MusicId,
                ])
                .update_columns([
                    models::score::Column::Score,
                    models::score::Column::Cool,
                    models::score::Column::Good,
                    models::score::Column::Bad,
                    models::score::Column::Miss,
                    models::score::Column::MaxCombo,
                    models::score::Column::JamCombo,
                    models::score::Column::Rate,
                    models::score::Column::Skills,
                    models::score::Column::Timestamp,
                ])
                .value(
                    models::score::Column::Score,
                    Expr::case(
                        Expr::col((models::score::Entity, models::score::Column::Score))
                            .gt(Expr::col(models::score::Column::Score)),
                        Expr::col((models::score::Entity, models::score::Column::Score)),
                    )
                    .finally(Expr::col(models::score::Column::Score)),
                )
                .to_owned(),
            )
            .exec(&self.connection)
            .await?;

        Ok(())
    }

    pub async fn get_user_scores(&self, user_id: u64) -> Vec<Score> {
        models::score::Entity::find()
            .filter(models::score::Column::UserId.eq(user_id))
            .all(&self.connection)
            .await
            .expect("Failed to query user scores")
            .into_iter()
            .map(|model| Score {
                id: model.id,
                user_id: model.user_id,
                music_id: model.music_id,
                score: model.score,
                cool: model.cool as u32,
                good: model.good as u32,
                bad: model.bad as u32,
                miss: model.miss as u32,
                max_combo: model.max_combo as u32,
                jam_combo: model.jam_combo as u32,
                timing: model.timing,
                rate: model.rate,
                fln: model.fln,
                sln: model.sln,
                nln: model.nln,
                arragement: model
                    .arragement
                    .chars()
                    .map(|c| c.to_digit(10).unwrap_or(0) as u8)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap_or([0; 7]),
                skills: model
                    .skills
                    .split(',')
                    .map(|s| u32::from_str_radix(s, 10).unwrap_or(0))
                    .collect(),
                timestamp: model.timestamp,
            })
            .collect()
    }

    pub async fn get_music_scores(&self, music_id: u32) -> Vec<Score> {
        models::score::Entity::find()
            .filter(models::score::Column::MusicId.eq(music_id))
            .all(&self.connection)
            .await
            .expect("Failed to query music scores")
            .into_iter()
            .map(|model| Score {
                id: model.id,
                user_id: model.user_id,
                music_id: model.music_id,
                score: model.score,
                cool: model.cool as u32,
                good: model.good as u32,
                bad: model.bad as u32,
                miss: model.miss as u32,
                max_combo: model.max_combo as u32,
                jam_combo: model.jam_combo as u32,
                timing: model.timing,
                rate: model.rate,
                fln: model.fln,
                sln: model.sln,
                nln: model.nln,
                arragement: model
                    .arragement
                    .chars()
                    .map(|c| c.to_digit(10).unwrap_or(0) as u8)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap_or([0; 7]),
                skills: model
                    .skills
                    .split(',')
                    .map(|s| u32::from_str_radix(s, 10).unwrap_or(0))
                    .collect(),
                timestamp: model.timestamp,
            })
            .collect()
    }

    pub async fn get_top_scores(&self, music_id: u32, limit: usize) -> Vec<Score> {
        models::score::Entity::find()
            .filter(models::score::Column::MusicId.eq(music_id))
            .order_by_desc(models::score::Column::Score)
            .limit(limit as u64)
            .all(&self.connection)
            .await
            .expect("Failed to query top scores")
            .into_iter()
            .map(|model| Score {
                id: model.id,
                user_id: model.user_id,
                music_id: model.music_id,
                score: model.score,
                cool: model.cool as u32,
                good: model.good as u32,
                bad: model.bad as u32,
                miss: model.miss as u32,
                max_combo: model.max_combo as u32,
                jam_combo: model.jam_combo as u32,
                timing: model.timing,
                rate: model.rate,
                fln: model.fln,
                sln: model.sln,
                nln: model.nln,
                arragement: model
                    .arragement
                    .chars()
                    .map(|c| c.to_digit(10).unwrap_or(0) as u8)
                    .collect::<Vec<u8>>()
                    .try_into()
                    .unwrap_or([0; 7]),
                skills: model
                    .skills
                    .split(',')
                    .map(|s| u32::from_str_radix(s, 10).unwrap_or(0))
                    .collect(),
                timestamp: model.timestamp,
            })
            .collect()
    }

    pub async fn remove_score(&self, score_id: u32) -> bool {
        let delete_result = models::score::Entity::delete_by_id(score_id)
            .exec(&self.connection)
            .await
            .expect("Failed to delete score");

        delete_result.rows_affected > 0
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let db = GameDatabase::new("mariadb://root:@localhost:3306/otwotest").await;
        assert!(db.connection.ping().await.is_ok());
    }

    #[tokio::test]
    async fn test_score_submittion() {
        let db = GameDatabase::new("mariadb://root:@localhost:3306/otwotest").await;

        let score1 = Score {
            id: 0,
            user_id: u32::MAX as u64,
            music_id: 1,
            score: 1000,
            cool: 10,
            good: 5,
            bad: 2,
            miss: 1,
            max_combo: 15,
            jam_combo: 3,
            timing: 0,
            rate: 1.0,
            fln: 0,
            sln: 0,
            nln: 0,
            arragement: [1, 2, 3, 4, 5, 6, 7],
            skills: vec![1, 2, 3],
            timestamp: chrono::Utc::now(),
        };

        let score2 = Score {
            id: 0,
            user_id: score1.user_id,
            music_id: 1,
            score: 1200, // Higher score
            cool: 12,
            good: 4,
            bad: 1,
            miss: 0,
            max_combo: 17,
            jam_combo: 4,
            timing: 0,
            rate: 1.5,
            fln: 0,
            sln: 0,
            nln: 0,
            arragement: [7, 6, 5, 4, 3, 2, 1],
            skills: vec![2, 3, 4],
            timestamp: chrono::Utc::now(),
        };

        // Submit the first score
        db.submit_scores(&[score1.clone()]).await.unwrap();

        // Submit the second score (higher)
        db.submit_scores(&[score2.clone()]).await.unwrap();

        // Retrieve the scores for the user and music
        let scores = db.get_user_scores(score1.user_id).await;

        // There should be only one score for this user and music, which is the higher one
        assert_eq!(scores.len(), 1);
        assert_eq!(scores[0].score, score2.score);

        // delete the score
        let deleted = db.remove_score(scores[0].id).await;
        assert!(deleted);
    }
}

use crate::{
    helpers::thing_helpers::create_user_thing,
    models::user::{Badge, UserProfile},
    Error,
};
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

pub struct UserService;

impl UserService {
    pub async fn get_user_profile(db: &Surreal<Any>, user_id: &str) -> Result<UserProfile, Error> {
        let user_thing = create_user_thing(user_id);

        let sql_query = "SELECT badges, created_at, experience_points, favorite_count, id, level, listen_count, listening_streak, total_listening_time, username FROM user WHERE id = $user_id;";

        let mut response = db
            .query(sql_query)
            .bind(("user_id", user_thing))
            .await
            .map_err(|e| {
                Error::DbError(format!("Erreur lors de l'exécution de la requête: {}", e))
            })?;

        let user_profile: Option<UserProfile> = response.take(0).map_err(|e| {
            Error::DbError(format!(
                "Erreur lors de la désérialisation du profil utilisateur: {}",
                e
            ))
        })?;

        match user_profile {
            Some(profile) => Ok(profile),
            None => Err(Error::DbError(format!(
                "Utilisateur introuvable avec l'ID: {}",
                user_id
            ))),
        }
    }
}

async fn add_badge_to_user(db: &Surreal<Any>, user_id: Thing, badge: Badge) -> Result<(), Error> {
    let badge_str = badge.to_string();

    let sql = "UPDATE $user_id SET badges += $badge";

    db.query(sql)
        .bind(("user_id", user_id))
        .bind(("badge", badge_str))
        .await?;

    Ok(())
}

//TODO: After each listen, etc., check if the user unlocks a badge
// If unlocked, return the info to the frontend

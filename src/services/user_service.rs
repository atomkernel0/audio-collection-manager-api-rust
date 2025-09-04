use crate::{
    auth::password_service,
    helpers::thing_helpers::create_user_thing,
    models::user::{UserProfile, UserRecord},
    Error,
};
use surrealdb::{engine::any::Any, Surreal};

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

    pub async fn check_if_username_is_available(
        db: &Surreal<Any>,
        username: &str,
    ) -> Result<bool, Error> {
        let sql_query = "SELECT * FROM user WHERE username = $username";

        let mut result = db
            .query(sql_query)
            .bind(("username", username.to_string()))
            .await?;

        let user: Option<UserRecord> = result.take(0)?;

        if user.is_none() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn update_username(
        db: &Surreal<Any>,
        user_id: &str,
        username: &str,
    ) -> Result<UserProfile, Error> {
        let user_thing = create_user_thing(user_id);

        let current = Self::get_user_profile(db, user_id).await?;
        if current.username == username {
            return Ok(current);
        }

        let mut check_res = db
            .query("SELECT * FROM user WHERE username = $username AND id != $user LIMIT 1")
            .bind(("username", username.to_string()))
            .bind(("user", user_thing.clone()))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la vérification du nom d'utilisateur: {}",
                    e
                ))
            })?;

        let conflict: Option<UserRecord> = check_res
            .take(0)
            .map_err(|e| Error::DbError(format!("Erreur lors du parsing du résultat: {}", e)))?;

        if conflict.is_some() {
            return Err(Error::UserAlreadyExists {
                username: username.to_string(),
            });
        }

        db.query("UPDATE $user SET username = $username")
            .bind(("user", user_thing))
            .bind(("username", username.to_string()))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la mise à jour du nom d'utilisateur: {}",
                    e
                ))
            })?;

        Self::get_user_profile(db, user_id).await
    }

    pub async fn change_password(
        db: &Surreal<Any>,
        user_id: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<bool, Error> {
        let user_thing = create_user_thing(user_id);

        // Fetch the current user (including password hash)
        let mut res = db
            .query("SELECT * FROM user WHERE id = $user LIMIT 1")
            .bind(("user", user_thing.clone()))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la récupération de l'utilisateur: {}",
                    e
                ))
            })?;

        let user: Option<UserRecord> = res.take(0).map_err(|e| {
            Error::DbError(format!(
                "Erreur lors du parsing du résultat utilisateur: {}",
                e
            ))
        })?;

        let user = user.ok_or_else(|| {
            Error::DbError(format!(
                "Utilisateur introuvable avec l'ID: {}",
                user_id
            ))
        })?;

        // Verify current password with bcrypt
        let valid = password_service
            ::verify_password(current_password, &user.password)
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur de vérification du mot de passe: {}",
                    e
                ))
            })?;

        if !valid {
            return Err(Error::InvalidPassword);
        }

        // Hash new password
        let hashed = password_service::hash_password(new_password).map_err(|e| {
            Error::DbError(format!(
                "Erreur lors du hash du mot de passe: {}",
                e
            ))
        })?;

        // Perform update
        db.query("UPDATE $user SET password = $password")
            .bind(("user", user_thing))
            .bind(("password", hashed))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la mise à jour du mot de passe: {}",
                    e
                ))
            })?;

        Ok(true)
    }

    pub async fn delete_user(db: &Surreal<Any>, user_id: &str) -> Result<bool, Error> {
        let user_thing = create_user_thing(user_id);

        let mut res = db
            .query("DELETE $user")
            .bind(("user", user_thing))
            .await
            .map_err(|e| {
                Error::DbError(format!(
                    "Erreur lors de la suppression de l'utilisateur: {}",
                    e
                ))
            })?;

        let deleted: Option<UserRecord> = res.take(0).map_err(|e| {
            Error::DbError(format!(
                "Erreur lors du parsing du résultat de suppression: {}",
                e
            ))
        })?;

        Ok(deleted.is_some())
    }
}
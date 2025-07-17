use crate::{
    auth::{jwt_service, password_service},
    error::{Error, Result},
    models::user::User,
};
use chrono::Utc;
use surrealdb::{engine::any::Any, Surreal};

pub async fn register_user(db: &Surreal<Any>, username: String, password: String) -> Result<User> {
    let sql = "SELECT * FROM user WHERE username = $username";
    let mut result = db.query(sql).bind(("username", username.clone())).await?;
    let user: Option<User> = result.take(0)?;
    if user.is_none() {
        let hashed_password =
            password_service::hash_password(&password).map_err(|_| Error::TokenCreationError)?;
        let new_user = User {
            id: None,
            username: username.clone(),
            password: hashed_password,
            created_at: Utc::now().into(),
            listen_count: 0,
            total_listening_time: 0,
            favorite_count: 0,
            listening_streak: 0,
            badges: vec![],
            level: 1,
            experience_points: 0,
        };
        db.create("user")
            .content(new_user)
            .await?
            .ok_or(Error::DbError("Could not create user".into()))
    } else {
        Err(Error::UserAlreadyExists { username })
    }
}

pub async fn login_user(db: &Surreal<Any>, username: String, password: String) -> Result<String> {
    let sql = "SELECT * FROM user WHERE username = $username";
    let mut result = db.query(sql).bind(("username", username.clone())).await?;
    let user: Option<User> = result.take(0)?;
    let user = user.ok_or_else(|| Error::UserNotFound { username })?;
    if !password_service::verify_password(&password, &user.password)? {
        return Err(Error::InvalidPassword);
    }
    let token = jwt_service::create_token(&user.id.clone().unwrap().to_string())?;
    Ok(token)
}

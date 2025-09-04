use crate::{
    auth::{
        password_service,
        token_service::{AuthConfig, TokenService},
    },
    error::{Error, Result},
    helpers::thing_helpers::{parse_id_part, thing_to_string},
    models::user::UserRecord,
};
use chrono::Utc;
use surrealdb::{engine::any::Any, Surreal};

pub struct AuthService;

impl AuthService {
    pub async fn register_user(
        db: &Surreal<Any>,
        username: String,
        password: String,
    ) -> Result<UserRecord> {
        if username.is_empty() || username.len() > 30 || username.contains(' ') {
            return Err(Error::InvalidUsername);
        }

        let sql = "SELECT * FROM user WHERE username = $username";
        let mut result = db.query(sql).bind(("username", username.clone())).await?;
        let user: Option<UserRecord> = result.take(0)?;
        if user.is_none() {
            let hashed_password = password_service::hash_password(&password)?;
            let new_user = UserRecord {
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

    pub async fn login_user(
        db: &Surreal<Any>,
        config: &AuthConfig,
        username: String,
        password: String,
    ) -> Result<String> {
        let sql = "SELECT * FROM user WHERE username = $username";
        let result: Option<UserRecord> = db
            .query(sql)
            .bind(("username", username.clone()))
            .await?
            .take(0)?;

        let user: UserRecord = result.ok_or_else(|| Error::UserNotFound {
            username: username.to_owned(),
        })?;

        if !password_service::verify_password(&password, &user.password)? {
            return Err(Error::InvalidPassword);
        }

        let token = match &user.id {
            Some(id) => {
                let thing_str = thing_to_string(id);
                let id_part = parse_id_part(&thing_str);
                TokenService::create_token(id_part.to_string(), config)?
            }
            None => return Err(Error::LoginFail),
        };

        Ok(token)
    }
}

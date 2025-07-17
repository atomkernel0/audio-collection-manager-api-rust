use crate::{helpers::album_helpers::album_exists, models::user::Badge, Error};
use serde::Deserialize;
use surrealdb::{engine::any::Any, sql::Thing, Surreal};

pub async fn add_badge_to_user(
    db: &Surreal<Any>,
    user_id: Thing,
    badge: Badge,
) -> Result<(), Error> {
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

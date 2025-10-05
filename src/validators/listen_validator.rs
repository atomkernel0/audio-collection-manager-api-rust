use serde::{Deserialize, Serialize};
use std::time::Duration as StdDuration;
use surrealdb::{engine::any::Any, Surreal};

use crate::{
    helpers::thing_helpers::{create_album_thing, create_song_thing, create_user_thing},
    Error
};

pub struct ListenValidator;

impl ListenValidator {
    /// Validates if a listen request is legitimate
    pub async fn validate_listen(
        db: &Surreal<Any>,
        song_id: &str,
        user_id: Option<&str>,
        client_ip: Option<&str>,
        song_duration_secs: u64,
    ) -> Result<ValidationResult, Error> {
        if user_id.is_none() {
            return Self::validate_anonymous_listen(db, client_ip, song_id).await;
        }

        let user_id = user_id.unwrap();

        let user_thing = create_user_thing(user_id);
        let song_thing = create_song_thing(song_id);

        let last_listen_check = r#"
            SELECT 
                last_listened_at,
                total_listens
            FROM user_listens_song 
            WHERE in = $user_id AND out = $song_id
            LIMIT 1
        "#;

        let mut response = db
            .query(last_listen_check)
            .bind(("user_id", user_thing.clone()))
            .bind(("song_id", song_thing))
            .await?;

        #[derive(Deserialize)]
        struct LastListenCheck {
            last_listened_at: Option<chrono::DateTime<chrono::Utc>>,
            #[allow(dead_code)]
            total_listens: Option<u64>,
        }

        let last_listen: Option<LastListenCheck> = response.take(0)?;

        if let Some(last_listen) = last_listen {
            if let Some(last_time) = last_listen.last_listened_at {
                let now = chrono::Utc::now();
                let time_since_last = now.signed_duration_since(last_time);
                
                // Rule 1: Can't listen to same song twice within 70% of its duration - 5s
                let min_interval_secs = ((0.7 * song_duration_secs as f64) as u64)
                    .saturating_sub(5)
                    .max(6);
                let min_interval = StdDuration::from_secs(min_interval_secs);
                
                if time_since_last < chrono::Duration::from_std(min_interval).unwrap() {
                    let elapsed_secs = time_since_last.num_seconds().max(0) as u64;
                    let retry_after = min_interval_secs.saturating_sub(elapsed_secs);
                    
                    return Ok(ValidationResult::RateLimited {
                        reason: "Too soon since last listen".to_string(),
                        retry_after_secs: retry_after,
                    });
                }
            }
        }

        let rate_limit_check = r#"
            LET $one_hour_ago = time::now() - 1h;
            LET $one_minute_ago = time::now() - 1m;
            
            LET $recent_listens = (
                SELECT count() as total FROM user_listens_song 
                WHERE in = $user_id 
                AND last_listened_at > $one_hour_ago
            );
            
            LET $very_recent = (
                SELECT count() as total FROM user_listens_song 
                WHERE in = $user_id 
                AND last_listened_at > $one_minute_ago
            );
            
            RETURN {
                hour_count: $recent_listens[0].total OR 0,
                minute_count: $very_recent[0].total OR 0
            };
        "#;

        let mut rate_response = db
            .query(rate_limit_check)
            .bind(("user_id", user_thing))
            .await?;

        #[derive(Deserialize)]
        struct RateLimitCheck {
            hour_count: u32,
            minute_count: u32,
        }

        let rates: Option<RateLimitCheck> = rate_response.take(0)?;

        if let Some(rates) = rates {
            // Rule 2: Max 100 listens per hour
            const MAX_LISTENS_PER_HOUR: u32 = 100;
            if rates.hour_count >= MAX_LISTENS_PER_HOUR {
                return Ok(ValidationResult::RateLimited {
                    reason: "Hourly rate limit exceeded".to_string(),
                    retry_after_secs: 3600,
                });
            }

            // Rule 3: Max 10 listens per minute (prevents rapid spam)
            const MAX_LISTENS_PER_MINUTE: u32 = 10;
            if rates.minute_count >= MAX_LISTENS_PER_MINUTE {
                return Ok(ValidationResult::RateLimited {
                    reason: "Rate limit exceeded".to_string(),
                    retry_after_secs: 60,
                });
            }
        }

        Ok(ValidationResult::Allowed)
    }

    /// Validates listen for anonymous users using IP-based rate limiting
    async fn validate_anonymous_listen(
        db: &Surreal<Any>,
        client_ip: Option<&str>,
        song_id: &str,
    ) -> Result<ValidationResult, Error> {
        let Some(ip) = client_ip else {
            return Ok(ValidationResult::Allowed);
        };

        let ip_owned = ip.to_string();
        let song_id_owned = song_id.to_string();

        let count_query = r#"
            SELECT count() AS total FROM anonymous_listen_log
            WHERE ip_address = $ip_address
            AND listened_at > time::now() - 1m
            GROUP ALL;
        "#;

        let mut count_response = db
            .query(count_query)
            .bind(("ip_address", ip_owned.clone()))
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            total: u64,
        }

        let count_result: Option<CountResult> = count_response.take(0)?;
        let minute_count = count_result.map(|r| r.total).unwrap_or(0);

        const MAX_ANONYMOUS_LISTENS_PER_MINUTE: u32 = 5;
        if minute_count >= MAX_ANONYMOUS_LISTENS_PER_MINUTE as u64 {
            return Ok(ValidationResult::RateLimited {
                reason: "Anonymous rate limit exceeded. Stop spamming.".to_string(),
                retry_after_secs: 60,
            });
        }

        let log_query = r#"
            CREATE anonymous_listen_log SET
                ip_address = $ip_address,
                song_id = $song_id,
                listened_at = time::now()
        "#;

        db.query(log_query)
            .bind(("ip_address", ip_owned))
            .bind(("song_id", song_id_owned))
            .await?;

        Ok(ValidationResult::Allowed)
    }

    /// Validates if an album listen request is legitimate
    pub async fn validate_album_listen(
        db: &Surreal<Any>,
        album_id: &str,
        user_id: Option<&str>,
        client_ip: Option<&str>,
        album_duration_secs: u64,
    ) -> Result<ValidationResult, Error> {
        if user_id.is_none() {
            return Self::validate_anonymous_album_listen(db, client_ip, album_id).await;
        }

        let user_id = user_id.unwrap();

        let user_thing = create_user_thing(user_id);
        let album_thing = create_album_thing(album_id);

        let last_listen_check = r#"
            SELECT
                last_listened_at,
                total_listens
            FROM user_listens_album
            WHERE in = $user_id AND out = $album_id
            LIMIT 1
        "#;

        let mut response = db
            .query(last_listen_check)
            .bind(("user_id", user_thing.clone()))
            .bind(("album_id", album_thing))
            .await?;

        #[derive(Deserialize)]
        struct LastListenCheck {
            last_listened_at: Option<chrono::DateTime<chrono::Utc>>,
            #[allow(dead_code)]
            total_listens: Option<u64>,
        }

        let last_listen: Option<LastListenCheck> = response.take(0)?;

        if let Some(last_listen) = last_listen {
            if let Some(last_time) = last_listen.last_listened_at {
                let now = chrono::Utc::now();
                let time_since_last = now.signed_duration_since(last_time);
                
                // Rule 1: Can't listen to same album twice within 70% of its duration - 5s
                let min_interval_secs = ((0.7 * album_duration_secs as f64) as u64)
                    .saturating_sub(5)
                    .max(6);
                let min_interval = StdDuration::from_secs(min_interval_secs);
                
                if time_since_last < chrono::Duration::from_std(min_interval).unwrap() {
                    let elapsed_secs = time_since_last.num_seconds().max(0) as u64;
                    let retry_after = min_interval_secs.saturating_sub(elapsed_secs);
                    
                    return Ok(ValidationResult::RateLimited {
                        reason: "Too soon since last listen".to_string(),
                        retry_after_secs: retry_after,
                    });
                }
            }
        }

        let rate_limit_check = r#"
            LET $one_hour_ago = time::now() - 1h;
            LET $one_minute_ago = time::now() - 1m;
            
            LET $recent_listens = (
                SELECT count() as total FROM user_listens_album
                WHERE in = $user_id
                AND last_listened_at > $one_hour_ago
            );
            
            LET $very_recent = (
                SELECT count() as total FROM user_listens_album
                WHERE in = $user_id
                AND last_listened_at > $one_minute_ago
            );
            
            RETURN {
                hour_count: $recent_listens[0].total OR 0,
                minute_count: $very_recent[0].total OR 0
            };
        "#;

        let mut rate_response = db
            .query(rate_limit_check)
            .bind(("user_id", user_thing))
            .await?;

        #[derive(Deserialize)]
        struct RateLimitCheck {
            hour_count: u32,
            minute_count: u32,
        }

        let rates: Option<RateLimitCheck> = rate_response.take(0)?;

        if let Some(rates) = rates {
            // Rule 2: Max 100 album listens per hour
            const MAX_LISTENS_PER_HOUR: u32 = 100;
            if rates.hour_count >= MAX_LISTENS_PER_HOUR {
                return Ok(ValidationResult::RateLimited {
                    reason: "Hourly rate limit exceeded".to_string(),
                    retry_after_secs: 3600,
                });
            }

            // Rule 3: Max 10 album listens per minute (prevents rapid spam)
            const MAX_LISTENS_PER_MINUTE: u32 = 10;
            if rates.minute_count >= MAX_LISTENS_PER_MINUTE {
                return Ok(ValidationResult::RateLimited {
                    reason: "Rate limit exceeded".to_string(),
                    retry_after_secs: 60,
                });
            }
        }

        Ok(ValidationResult::Allowed)
    }

    /// Validates album listen for anonymous users using IP-based rate limiting
    async fn validate_anonymous_album_listen(
        db: &Surreal<Any>,
        client_ip: Option<&str>,
        album_id: &str,
    ) -> Result<ValidationResult, Error> {
        let Some(ip) = client_ip else {
            return Ok(ValidationResult::Allowed);
        };

        let ip_owned = ip.to_string();
        let album_id_owned = album_id.to_string();

        let count_query = r#"
            SELECT count() AS total FROM anonymous_listen_log
            WHERE ip_address = $ip_address
            AND listened_at > time::now() - 1m
            GROUP ALL;
        "#;

        let mut count_response = db
            .query(count_query)
            .bind(("ip_address", ip_owned.clone()))
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            total: u64,
        }

        let count_result: Option<CountResult> = count_response.take(0)?;
        let minute_count = count_result.map(|r| r.total).unwrap_or(0);

        const MAX_ANONYMOUS_LISTENS_PER_MINUTE: u32 = 5;
        if minute_count >= MAX_ANONYMOUS_LISTENS_PER_MINUTE as u64 {
            return Ok(ValidationResult::RateLimited {
                reason: "Anonymous rate limit exceeded. Stop spamming.".to_string(),
                retry_after_secs: 60,
            });
        }

        let log_query = r#"
            CREATE anonymous_listen_log SET
                ip_address = $ip_address,
                album_id = $album_id,
                listened_at = time::now()
        "#;

        db.query(log_query)
            .bind(("ip_address", ip_owned))
            .bind(("album_id", album_id_owned))
            .await?;

        Ok(ValidationResult::Allowed)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ValidationResult {
    Allowed,
    RateLimited {
        reason: String,
        retry_after_secs: u64
    },
}
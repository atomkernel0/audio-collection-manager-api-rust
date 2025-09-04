use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use surrealdb::{engine::any::Any, sql::Thing, Surreal};
use crate::{models::{database_helpers::CountResult, user::{BadgeEnum, UserRecord}}, Error, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BadgeUnlockResult {
    pub new_badges: Vec<BadgeEnum>,
    pub message: Option<String>,
}

pub struct BadgeService;

impl BadgeService {
    /// Vérifie et attribue les badges après une écoute
    pub async fn check_badges_after_listen(
        db: &Surreal<Any>,
        user_id: Thing,
    ) -> Result<BadgeUnlockResult> {
        // Récupérer l'utilisateur avec ses stats actuelles
        let mut response = db.query("SELECT * FROM user WHERE id = $id").bind(("id", user_id.to_string())).await?;
        let user: Option<UserRecord> = response.take(0)?;
        
        let user = user.ok_or_else(|| Error::UserNotFound { username: user_id.to_string() })?;
        
        // Convertir les badges existants en HashSet
        let existing_badges: HashSet<BadgeEnum> = user.badges.into_iter().collect();
        let mut new_badges = Vec::new();
        
        // Vérifier les badges de temps d'écoute (en secondes)
        let listening_hours = user.total_listening_time / 3600;
        
        // Badge 10 heures
        if listening_hours >= 10 && !existing_badges.contains(&BadgeEnum::Listen10Hours) {
            new_badges.push(BadgeEnum::Listen10Hours);
        }
        // Badge 50 heures
        if listening_hours >= 50 && !existing_badges.contains(&BadgeEnum::Listen50Hours) {
            new_badges.push(BadgeEnum::Listen50Hours);
        }
        // Badge 100 heures
        if listening_hours >= 100 && !existing_badges.contains(&BadgeEnum::Listen100Hours) {
            new_badges.push(BadgeEnum::Listen100Hours);
        }
        // Badge 500 heures
        if listening_hours >= 500 && !existing_badges.contains(&BadgeEnum::Listen500Hours) {
            new_badges.push(BadgeEnum::Listen500Hours);
        }
        // Badge 1000 heures
        if listening_hours >= 1000 && !existing_badges.contains(&BadgeEnum::Listen1000Hours) {
            new_badges.push(BadgeEnum::Listen1000Hours);
        }
        
        // Sauvegarder les nouveaux badges si nécessaire
        if !new_badges.is_empty() {
            Self::add_badges_to_user(db, user_id, &new_badges).await?;
        }
        
        let message = if !new_badges.is_empty() {
            Some(format!(
                "🎉 Félicitations ! Vous avez débloqué {} nouveau(x) badge(s) !",
                new_badges.len()
            ))
        } else {
            None
        };
        
        Ok(BadgeUnlockResult {
            new_badges,
            message,
        })
    }
    
    /// Vérifie les badges après ajout d'un favori
    pub async fn check_badges_after_favorite(
        db: &Surreal<Any>,
        user_id: Thing,
    ) -> Result<BadgeUnlockResult> {
        let mut response = db.query("SELECT * FROM user WHERE id = $id").bind(("id", user_id.to_string())).await?;
        let user: Option<UserRecord> = response.take(0)?;

        let user = user.ok_or_else(|| Error::UserNotFound { username: user_id.to_string() })?;
        
        let existing_badges: HashSet<BadgeEnum> = user.badges.into_iter().collect();
        let mut new_badges = Vec::new();
        
        // Vérifier les badges de favoris
        let fav_count = user.favorite_count;
        
        if fav_count >= 10 && !existing_badges.contains(&BadgeEnum::Favorite10Song) {
            new_badges.push(BadgeEnum::Favorite10Song);
        }
        if fav_count >= 20 && !existing_badges.contains(&BadgeEnum::Favorite20Song) {
            new_badges.push(BadgeEnum::Favorite20Song);
        }
        if fav_count >= 50 && !existing_badges.contains(&BadgeEnum::Favorite50Song) {
            new_badges.push(BadgeEnum::Favorite50Song);
        }
        if fav_count >= 100 && !existing_badges.contains(&BadgeEnum::Favorite100Song) {
            new_badges.push(BadgeEnum::Favorite100Song);
        }
        if fav_count >= 200 && !existing_badges.contains(&BadgeEnum::Favorite200Song) {
            new_badges.push(BadgeEnum::Favorite200Song);
        }
        
        if !new_badges.is_empty() {
            Self::add_badges_to_user(db, user_id, &new_badges).await?;
        }
        
        let message = if !new_badges.is_empty() {
            Some(format!(
                "🏆 Nouveau badge débloqué pour vos favoris !",
            ))
        } else {
            None
        };
        
        Ok(BadgeUnlockResult {
            new_badges,
            message,
        })
    }
    
    /// Vérifie les badges après ajout à une playlist
    pub async fn check_badges_after_playlist_add(
        db: &Surreal<Any>,
        user_id: Thing,
    ) -> Result<BadgeUnlockResult> {
        // Compter le nombre total de chansons ajoutées aux playlists par cet utilisateur
        let query = "
            SELECT count() as total 
            FROM playlist_contains_song 
            WHERE added_by = $user_id 
            GROUP ALL
        ";
        
        let mut response = db.query(query)
            .bind(("user_id", user_id.clone()))
            .await?;
            
        let count: Option<CountResult> = response.take(0)?;
        let playlist_songs = count.map(|c| c.total).unwrap_or(0);
        
        // Récupérer les badges existants
        let mut response = db.query("SELECT * FROM user WHERE id = $id").bind(("id", user_id.to_string())).await?;
        let user: Option<UserRecord> = response.take(0)?;
        let user = user.ok_or_else(|| Error::UserNotFound { username: user_id.to_string() })?;

        let existing_badges: HashSet<BadgeEnum> = user.badges.into_iter().collect();
        
        let mut new_badges = Vec::new();
        
        if playlist_songs >= 10 && !existing_badges.contains(&BadgeEnum::Playlist10Song) {
            new_badges.push(BadgeEnum::Playlist10Song);
        }
        if playlist_songs >= 30 && !existing_badges.contains(&BadgeEnum::Playlist30Song) {
            new_badges.push(BadgeEnum::Playlist30Song);
        }
        if playlist_songs >= 70 && !existing_badges.contains(&BadgeEnum::Playlist70Song) {
            new_badges.push(BadgeEnum::Playlist70Song);
        }
        if playlist_songs >= 150 && !existing_badges.contains(&BadgeEnum::Playlist150Song) {
            new_badges.push(BadgeEnum::Playlist150Song);
        }
        if playlist_songs >= 250 && !existing_badges.contains(&BadgeEnum::Playlist250Song) {
            new_badges.push(BadgeEnum::Playlist250Song);
        }
        
        if !new_badges.is_empty() {
            Self::add_badges_to_user(db, user_id, &new_badges).await?;
        }
        
        let message = if !new_badges.is_empty() {
            Some("🎵 Badge playlist débloqué !".to_string())
        } else {
            None
        };
        
        Ok(BadgeUnlockResult {
            new_badges,
            message,
        })
    }
    
    /// Ajoute des badges à un utilisateur
    async fn add_badges_to_user(
        db: &Surreal<Any>,
        user_id: Thing,
        badges: &[BadgeEnum],
    ) -> Result<()> {
        let badge_strings: Vec<String> = badges.iter().map(|b| b.to_string()).collect();
        
        // Utilise array::union pour éviter les doublons
        let query = "UPDATE $user_id SET badges = array::union(badges, $new_badges)";
        
        db.query(query)
            .bind(("user_id", user_id))
            .bind(("new_badges", badge_strings))
            .await?;
        
        Ok(())
    }
}
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    LoginFail,

    // -- Auth errors.
    AuthFailNoAuthTokenCookie,
    AuthFailTokenWrongFormat,
    AuthFailCtxNotInRequestExt,
    TokenCreationError(String),
    InvalidToken,
    InvalidUsername,

    EnvVarError(String),
    DbError(String),
    AlbumNotFound {
        id: String,
    },
    ArtistNotFound {
        id: String,
    },
    SongNotFound {
        id: String,
    },
    PlaylistNotFound {
        id: String,
    },
    UserAlreadyExists {
        username: String,
    },
    UserNotFound {
        username: String,
    },
    InvalidPassword,
    InvalidCaptcha,

    // -- Favorite errors.
    FavoriteAlreadyExists {
        item_type: String,
        item_id: String,
    },
    FavoriteNotFound {
        item_type: String,
        item_id: String,
    },
    SongAlreadyExistsInPlaylist {
        song_id: String,
        playlist_id: String,
    },
    InvalidFavoriteRequest {
        reason: String,
    },
    InvalidInput {
        reason: String,
    },
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status_code, client_error) = self.client_status_and_error();

        let response_body = serde_json::json!({
            "error": client_error.as_ref(),
            "details": self.to_string()
        });

        (status_code, Json(response_body)).into_response()
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        #[allow(unreachable_patterns)]
        match self {
            Self::LoginFail => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            Self::AuthFailNoAuthTokenCookie
            | Self::AuthFailTokenWrongFormat
            | Self::AuthFailCtxNotInRequestExt => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

            Self::TokenCreationError { .. } | Self::InvalidToken => {
                (StatusCode::INTERNAL_SERVER_ERROR, ClientError::TOKEN_ERROR)
            }

            Error::DbError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
            Error::AlbumNotFound { id: _ } => {
                (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND)
            }
            Error::ArtistNotFound { id: _ } => {
                (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND)
            }
            Error::SongNotFound { id: _ } => {
                (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND)
            }

            Error::UserAlreadyExists { username: _ } => {
                (StatusCode::CONFLICT, ClientError::USER_ALREADY_EXISTS)
            }

            Error::UserNotFound { .. } => (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND),
            Error::InvalidPassword | Error::InvalidUsername => (StatusCode::FORBIDDEN, ClientError::INVALID_CREDENTIALS),
            Error::InvalidCaptcha => (StatusCode::BAD_REQUEST, ClientError::INVALID_CAPTCHA),

            Error::FavoriteAlreadyExists { .. } | Error::SongAlreadyExistsInPlaylist { .. } => {
                (StatusCode::CONFLICT, ClientError::RESOURCE_ALREADY_EXISTS)
            }
            Error::FavoriteNotFound { .. } => {
                (StatusCode::NOT_FOUND, ClientError::RESOURCE_NOT_FOUND)
            }
            Error::InvalidFavoriteRequest { .. } | Error::InvalidInput { .. } => {
                (StatusCode::BAD_REQUEST, ClientError::INVALID_PARAMS)
            }

            // Fallback
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}

#[derive(Debug, strum_macros::AsRefStr)]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    INVALID_PARAMS,
    SERVICE_ERROR,
    RESOURCE_NOT_FOUND,
    RESOURCE_ALREADY_EXISTS,
    TOKEN_ERROR,
    USER_ALREADY_EXISTS,
    INVALID_CREDENTIALS,
    INVALID_CAPTCHA,
}

impl From<surrealdb::Error> for Error {
    fn from(err: surrealdb::Error) -> Self {
        Error::DbError(err.to_string())
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::EnvVarError(err.to_string())
    }
}

impl From<bcrypt::BcryptError> for Error {
    fn from(err: bcrypt::BcryptError) -> Self {
        Error::DbError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for Error {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Error::TokenCreationError(err.to_string())
    }
}

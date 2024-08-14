
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use axum::{extract::State, http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use sqlx::PgPool;
use sqlx::Error as SqlxError;

#[derive(Deserialize)]
pub struct Payload {
    email: String,
    password: String,
}
#[derive(Debug)]
pub enum LoginError {
    InvalidCredentials,
    MissingCredentials,
    DatabaseError,
    TokenCreation,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            LoginError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password",
            ),
            LoginError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
            ),
            LoginError::MissingCredentials => (
                StatusCode::BAD_REQUEST,
                "Missing email or password",
            ),
            LoginError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred while creating the token",
            ),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}

#[derive(Serialize)]
pub struct AccessToken {
    access_token: String,
    refresh_token: String,
}
impl AccessToken {
    fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }

}

#[derive(Serialize)]
struct Claims {
    sub: String,
    exp: String,
}

pub async fn login(State(pool): State<PgPool>, Json(payload): Json<Payload>) -> Result<Json<AccessToken>, LoginError> {
    let email = payload.email;
    let password = payload.password;

    if email.is_empty() || password.is_empty() {
        return Err(LoginError::MissingCredentials);
    }
    let db_password: String = sqlx::query_scalar("SELECT password FROM employee WHERE email = $1")
        .bind(&email)
        .fetch_one(&pool)
        .await
        .map_err(|err| match err {
          SqlxError::RowNotFound => LoginError::InvalidCredentials, // Mapping auf InvalidCredentials
          _ => LoginError::DatabaseError,                            // Mapping auf DatabaseError für alle anderen Fehler
        })?;

    if db_password != password {
        return Err(LoginError::InvalidCredentials);
    }

    let acc_claims = Claims {
      sub: email.clone(),
      exp: (Utc::now() + Duration::days(1)).to_rfc3339(),
    };
    let ref_claims = Claims {
      sub: email.clone(),
      exp: (Utc::now() + Duration::days(30)).to_rfc3339(),
    };

    let access_token = encode(&Header::default(), &acc_claims, &EncodingKey::from_secret(dotenvy::var("SECRET").expect("No secret was provided.").as_ref())).map_err(|_| LoginError::TokenCreation)?;
    let refresh_token = encode(&Header::default(), &ref_claims, &EncodingKey::from_secret(dotenvy::var("SECRET").expect("No secret was provided.").as_ref())).map_err(|_| LoginError::TokenCreation)?;
    Ok(Json(AccessToken::new(access_token, refresh_token)))
}

pub async fn refresh() {
    // TODO
}

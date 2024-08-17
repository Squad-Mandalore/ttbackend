use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

impl LoginResponse {
    fn new(access_token: String, refresh_token: String) -> Self {
        Self {
            access_token,
            refresh_token,
        }
    }
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String,
    exp: i64,
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
            LoginError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password")
            }
            LoginError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
            ),
            LoginError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing email or password")
            }
            LoginError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred while creating the token",
            ),
        };
        (status, error_message).into_response()
    }
}

pub async fn login(
    State(pool): State<PgPool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, LoginError> {
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
            SqlxError::RowNotFound => LoginError::InvalidCredentials,
            _ => LoginError::DatabaseError,
        })?;

    if db_password != password {
        return Err(LoginError::InvalidCredentials);
    }

    Ok(Json(create_login_response(email)?))
}

pub fn create_login_response(email: String) -> Result<LoginResponse, LoginError> {
    let acc_claims = Claims {
        sub: email.clone(),
        exp: (Utc::now() + Duration::days(1)).timestamp(),
    };
    let ref_claims = Claims {
        sub: email.clone(),
        exp: (Utc::now() + Duration::days(30)).timestamp(),
    };

    let access_token = encode(
        &Header::default(),
        &acc_claims,
        &EncodingKey::from_secret(
            dotenvy::var("SECRET")
                .expect("No secret was provided.")
                .as_ref(),
        ),
    )
    .map_err(|_| LoginError::TokenCreation)?;
    let refresh_token = encode(
        &Header::default(),
        &ref_claims,
        &EncodingKey::from_secret(
            dotenvy::var("SECRET")
                .expect("No secret was provided.")
                .as_ref(),
        ),
    )
    .map_err(|_| LoginError::TokenCreation)?;

    Ok(LoginResponse::new(access_token, refresh_token))
}

pub async fn refresh(
    Json(refresh_request): Json<RefreshRequest>,
) -> Result<Json<LoginResponse>, LoginError> {
    if refresh_request.refresh_token.is_empty() {
        return Err(LoginError::MissingCredentials);
    }

    let claims = jsonwebtoken::decode::<Claims>(
        &refresh_request.refresh_token,
        &DecodingKey::from_secret(
            dotenvy::var("SECRET")
                .expect("No secret was provided.")
                .as_ref(),
        ),
        &Validation::default(),
    )
    .map_err(|_| LoginError::InvalidCredentials)?;

    let email = claims.claims.sub;
    let mut logres = create_login_response(email)?;
    logres.refresh_token = refresh_request.refresh_token;
    Ok(Json(logres))
}

pub async fn auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());
    if let Some(auth_header) = auth_header {
        let token = auth_header.replace("Bearer ", "");
        let claims = jsonwebtoken::decode::<Claims>(
            &token,
            &DecodingKey::from_secret(
                dotenvy::var("SECRET")
                    .expect("No secret was provided.")
                    .as_ref(),
            ),
            &Validation::default(),
        )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
        request.extensions_mut().insert(claims.claims.sub);
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

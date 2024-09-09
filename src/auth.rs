use axum::{
    extract::{Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json, RequestExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use chrono::{prelude::*, Duration};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::Error as SqlxError;
use sqlx::PgPool;

use crate::security::verify_password;

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
#[serde(rename_all = "camelCase")]
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
    InvalidToken,
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let (status, error_message, www_authenticate) = match self {
            LoginError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "Invalid email or password",
                Some("Bearer realm=\"Application\", charset=\"UTF-8\""),
            ),
            LoginError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Invalid Token",
                Some("Bearer realm=\"Application\", charset=\"UTF-8\""),
            ),
            LoginError::DatabaseError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred",
                None,
            ),

            LoginError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing email or password", None)
            }
            LoginError::TokenCreation => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unexpected error occurred while creating the token",
                None,
            ),
        };
        let mut response = Response::new(error_message.into());
        *response.status_mut() = status;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        if let Some(authenticate) = www_authenticate {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(authenticate),
            );
        }
        response
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
    let account = sqlx::query!(
        r#"SELECT employee_id, password FROM employee WHERE email = $1"#,
        &email,
    )
    .fetch_one(&pool)
    .await
    .map_err(|err| match err {
        SqlxError::RowNotFound => LoginError::InvalidCredentials,
        _ => LoginError::DatabaseError,
    })?;

    if !verify_password(password, &pool, &account.employee_id)
        .await
        .map_err(|_| LoginError::DatabaseError)?
    {
        return Err(LoginError::InvalidCredentials);
    }

    Ok(Json(create_login_response(account.employee_id)?))
}

pub fn create_login_response(employee_id: i32) -> Result<LoginResponse, LoginError> {
    let acc_claims = Claims {
        sub: employee_id.to_string(),
        exp: (Utc::now() + Duration::days(1)).timestamp(),
    };
    let ref_claims = Claims {
        sub: employee_id.to_string(),
        exp: (Utc::now() + Duration::days(30)).timestamp(),
    };

    let jwt_secret = dotenvy::var("JWT_SECRET").expect("No secret was provided.");

    let access_token = encode(
        &Header::default(),
        &acc_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    )
    .map_err(|_| LoginError::TokenCreation)?;
    let refresh_token = encode(
        &Header::default(),
        &ref_claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
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
            dotenvy::var("JWT_SECRET")
                .expect("No secret was provided.")
                .as_ref(),
        ),
        &Validation::default(),
    )
    .map_err(|_| LoginError::InvalidCredentials)?;

    // assuming the sub is proper after validation I can 'safely' use unwrap here
    let employee_id: i32 = claims.claims.sub.parse::<i32>().unwrap();
    let mut logres = create_login_response(employee_id)?;
    logres.refresh_token = refresh_request.refresh_token;
    Ok(Json(logres))
}

pub async fn auth(mut request: Request, next: Next) -> Result<Response, LoginError> {
    let TypedHeader(Authorization(bearer)) = request
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| LoginError::InvalidToken)?;
    let claims = jsonwebtoken::decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_secret(
            dotenvy::var("JWT_SECRET")
                .expect("No secret was provided.")
                .as_ref(),
        ),
        &Validation::default(),
    )
    .map_err(|_| LoginError::InvalidCredentials)?;
    request.extensions_mut().insert(claims.claims.sub);
    Ok(next.run(request).await)
}

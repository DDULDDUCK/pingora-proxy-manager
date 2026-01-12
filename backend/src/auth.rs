use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    RequestPartsExt,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use std::env;

// 환경 변수에서 JWT Secret 가져오기
fn get_jwt_secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .unwrap_or_else(|_| "super_secret_key_change_me_in_production".to_string())
        .into_bytes()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,  // username
    pub exp: usize,   // 만료 시간
    pub user_id: i64, // 사용자 ID
    pub role: String, // 역할 (admin, operator, viewer)
}

// 역할 정의
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    Viewer = 0,   // 읽기 전용
    Operator = 1, // 호스트/스트림 관리 가능
    Admin = 2,    // 모든 권한 (사용자 관리 포함)
}

impl Role {
    pub fn from_str(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            "operator" => Role::Operator,
            _ => Role::Viewer,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Operator => "operator",
            Role::Viewer => "viewer",
        }
    }
}

impl Claims {
    pub fn role(&self) -> Role {
        Role::from_str(&self.role)
    }

    pub fn is_admin(&self) -> bool {
        self.role() == Role::Admin
    }

    pub fn can_manage_hosts(&self) -> bool {
        self.role() >= Role::Operator
    }

    pub fn can_manage_users(&self) -> bool {
        self.role() == Role::Admin
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid token"),
            AuthError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        let body = axum::Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

// 비밀번호 해싱
pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST)
}

// 비밀번호 검증
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

// JWT 발급
pub fn create_jwt(username: &str, user_id: i64, role: &str) -> Result<String, AuthError> {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize
        + 60 * 60 * 24; // 24시간 유효

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
        user_id,
        role: role.to_owned(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&get_jwt_secret()),
    )
    .map_err(|_| AuthError::TokenCreation)
}

// 이전 버전 호환용 (기본 admin 권한)
pub fn create_jwt_simple(username: &str) -> Result<String, AuthError> {
    create_jwt(username, 0, "admin")
}

// JWT 검증 (Axum Extractor)
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Bearer 토큰 추출
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthError::InvalidToken)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(&get_jwt_secret()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

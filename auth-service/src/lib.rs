use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Missing authorization header")]
    MissingToken,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Token expired")]
    TokenExpired,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub roles: Vec<String>,
    pub exp: usize,
    #[serde(rename = "type")]
    pub token_type: TokenType,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    User,
    System,
}

#[derive(Clone)]
pub struct AuthService {
    encoding_key: Arc<EncodingKey>,
    decoding_key: Arc<DecodingKey>,
}

impl AuthService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: Arc::new(EncodingKey::from_secret(secret)),
            decoding_key: Arc::new(DecodingKey::from_secret(secret)),
        }
    }

    pub fn create_token(
        &self,
        user_id: &str,
        roles: Vec<String>,
        token_type: TokenType,
        expires_in_hours: i64,
    ) -> Result<String, AuthError> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(expires_in_hours))
            .expect("valid timestamp")
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_owned(),
            roles,
            exp: expiration,
            token_type,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(|_| AuthError::InvalidToken)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|_| AuthError::InvalidToken)
    }

    pub fn has_role(&self, claims: &Claims, required_role: &str) -> bool {
        claims.roles.iter().any(|r| r == required_role)
    }

    pub fn has_any_role(&self, claims: &Claims, required_roles: &[String]) -> bool {
        required_roles
            .iter()
            .any(|required| claims.roles.iter().any(|r| r == required))
    }
}

// Middleware for user authentication
pub async fn user_auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token(&headers)?;

    let claims = auth_service
        .verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if claims.token_type != TokenType::User {
        return Err(StatusCode::UNAUTHORIZED);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

// Middleware for system-to-system authentication
pub async fn system_auth_middleware(
    State(auth_service): State<AuthService>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token(&headers)?;

    let claims = auth_service
        .verify_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if claims.token_type != TokenType::System {
        return Err(StatusCode::UNAUTHORIZED);
    }

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

// Role-based authorization middleware factory
pub fn require_roles(
    required_roles: Vec<String>,
) -> impl Fn(
    State<AuthService>,
    Request,
    Next,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>>
+ Clone {
    move |State(auth_service): State<AuthService>, request: Request, next: Next| {
        let required_roles = required_roles.clone();
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or(StatusCode::UNAUTHORIZED)?
                .clone();

            if !auth_service.has_any_role(&claims, &required_roles) {
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(request).await)
        })
    }
}

fn extract_token(headers: &HeaderMap) -> Result<String, StatusCode> {
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(auth_header.trim_start_matches("Bearer ").to_string())
}

// Helper struct for extracting claims in handlers
pub struct AuthUser(pub Claims);

impl<S> axum::extract::FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthUser)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_verify_user_token() {
        let auth_service = AuthService::new(b"test_secret");
        let token = auth_service
            .create_token("user123", vec!["user".to_string()], TokenType::User, 24)
            .unwrap();

        let claims = auth_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, TokenType::User);
        assert!(claims.roles.contains(&"user".to_string()));
    }

    #[test]
    fn test_create_and_verify_system_token() {
        let auth_service = AuthService::new(b"test_secret");
        let token = auth_service
            .create_token(
                "service-a",
                vec!["system".to_string()],
                TokenType::System,
                24,
            )
            .unwrap();

        let claims = auth_service.verify_token(&token).unwrap();
        assert_eq!(claims.sub, "service-a");
        assert_eq!(claims.token_type, TokenType::System);
    }

    #[test]
    fn test_role_checking() {
        let auth_service = AuthService::new(b"test_secret");
        let claims = Claims {
            sub: "user123".to_string(),
            roles: vec!["user".to_string(), "admin".to_string()],
            exp: 0,
            token_type: TokenType::User,
        };

        assert!(auth_service.has_role(&claims, "user"));
        assert!(auth_service.has_role(&claims, "admin"));
        assert!(!auth_service.has_role(&claims, "superadmin"));
    }
}

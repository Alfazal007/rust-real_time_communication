use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use super::generate_token::Claims;

pub fn validate_token(token: &str, secret: &str) -> Result<Claims, String> {
    let validation = Validation::new(Algorithm::HS256);
    let key = DecodingKey::from_secret(secret.as_bytes());
    match decode::<Claims>(token, &key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(err) => match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => Err("Token expired".to_string()),
            jsonwebtoken::errors::ErrorKind::InvalidToken => Err("Invalid token".to_string()),
            _ => Err("Token validation failed".to_string()),
        },
    }
}

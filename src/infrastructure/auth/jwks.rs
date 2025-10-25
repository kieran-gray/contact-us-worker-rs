use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jwt_compact::{
    AlgorithmExt, UntrustedToken,
    alg::{Rsa, RsaPublicKey},
    prelude::*,
};
use num_bigint_dig::BigUint;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{self, Display};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Jwk {
    pub kty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alg: Option<String>,
    pub n: String,
    pub e: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JWKS {
    pub keys: Vec<Jwk>,
}

impl JWKS {
    pub fn find(&self, kid: &str) -> Option<&Jwk> {
        self.keys
            .iter()
            .find(|jwk| jwk.kid.as_ref().map(|k| k == kid).unwrap_or(false))
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidToken(String),
    InvalidJWK(String),
    InvalidSignature,
    InvalidClaims(Vec<String>),
    Base64Error(String),
}

impl Error for ValidationError {}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidToken(msg) => write!(f, "Invalid token: {msg}"),
            ValidationError::InvalidJWK(msg) => write!(f, "Invalid JWK: {msg}"),
            ValidationError::InvalidSignature => f.write_str("JWT signature validation failed"),
            ValidationError::InvalidClaims(errs) => {
                write!(f, "Invalid claims: {}", errs.join(", "))
            }
            ValidationError::Base64Error(msg) => write!(f, "Base64 error: {msg}"),
        }
    }
}

type JWTResult<T> = Result<T, ValidationError>;

pub fn token_kid(token: &str) -> JWTResult<Option<String>> {
    let parts: Vec<&str> = token.splitn(2, '.').collect();
    if parts.len() < 2 {
        return Err(ValidationError::InvalidToken(
            "Invalid JWT format".to_string(),
        ));
    }

    let header_bytes = URL_SAFE_NO_PAD
        .decode(parts[0])
        .map_err(|e| ValidationError::Base64Error(e.to_string()))?;

    #[derive(Deserialize)]
    struct Header {
        kid: Option<String>,
    }

    let header: Header = serde_json::from_slice(&header_bytes)
        .map_err(|e| ValidationError::InvalidToken(e.to_string()))?;

    Ok(header.kid)
}

pub fn validate(
    token: &str,
    jwk: &Jwk,
    issuer: Option<&str>,
    _audience: Option<&str>,
    subject_required: bool,
) -> JWTResult<Value> {
    let n_bytes = URL_SAFE_NO_PAD
        .decode(&jwk.n)
        .map_err(|e| ValidationError::InvalidJWK(format!("Invalid n: {e}")))?;

    let e_bytes = URL_SAFE_NO_PAD
        .decode(&jwk.e)
        .map_err(|e| ValidationError::InvalidJWK(format!("Invalid e: {e}")))?;

    let n = BigUint::from_bytes_be(&n_bytes);
    let e = BigUint::from_bytes_be(&e_bytes);

    let public_key = RsaPublicKey::new(n, e)
        .map_err(|e| ValidationError::InvalidJWK(format!("Failed to create RSA key: {e}")))?;

    let untrusted =
        UntrustedToken::new(token).map_err(|e| ValidationError::InvalidToken(format!("{e}")))?;

    let token_data: Token<Value> = Rsa::rs256()
        .validator::<Value>(&public_key)
        .validate(&untrusted)
        .map_err(|_| ValidationError::InvalidSignature)?;

    let claims = token_data.claims().custom.clone();

    if let Some(expected_issuer) = issuer {
        if let Some(actual_issuer) = claims.get("iss").and_then(|v| v.as_str()) {
            if actual_issuer != expected_issuer {
                return Err(ValidationError::InvalidClaims(vec![
                    "Issuer mismatch".to_string(),
                ]));
            }
        } else {
            return Err(ValidationError::InvalidClaims(vec![
                "Missing issuer claim".to_string(),
            ]));
        }
    }

    if subject_required && claims.get("sub").is_none() {
        return Err(ValidationError::InvalidClaims(vec![
            "Subject claim required but missing".to_string(),
        ]));
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwks_find() {
        let jwks_json = r#"{
            "keys": [
                {
                    "kty": "RSA",
                    "kid": "test-key-1",
                    "n": "test",
                    "e": "AQAB"
                }
            ]
        }"#;

        let jwks: JWKS = serde_json::from_str(jwks_json).unwrap();
        assert!(jwks.find("test-key-1").is_some());
        assert!(jwks.find("non-existent").is_none());
    }
}

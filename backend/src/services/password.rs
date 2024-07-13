use std::fmt;

use argon2::{
    password_hash::{rand_core::OsRng, Error, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};

use crate::model::YakManPassword;

#[allow(dead_code)]
#[derive(Debug)]
pub struct PasswordHashError {
    pub inner: Box<Error>,
}

pub fn hash_password(password: &str) -> Result<String, PasswordHashError> {
    // Example from: https://docs.rs/argon2/latest/argon2
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| PasswordHashError { inner: Box::new(e) })?
        .to_string();

    return Ok(password_hash);
}

pub fn verify_password(
    password: &str,
    record: YakManPassword,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(&record.hash)?;
    return Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok());
}

#[derive(Debug, Clone, PartialEq)]
pub enum PasswordStrengthError {
    TooShort,
    TooLong,
    MissingUppercase,
    MissingLowercase,
    MissingDigit,
}

impl fmt::Display for PasswordStrengthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PasswordStrengthError::TooShort => {
                write!(f, "Password must be at least 8 characters long")
            }
            PasswordStrengthError::TooLong => {
                write!(f, "Password must be less than 100 characters long")
            }
            PasswordStrengthError::MissingUppercase => {
                write!(f, "Password must contain at least one uppercase letter")
            }
            PasswordStrengthError::MissingLowercase => {
                write!(f, "Password must contain at least one lowercase letter")
            }
            PasswordStrengthError::MissingDigit => {
                write!(f, "Password must contain at least one digit")
            }
        }
    }
}

pub fn validate_password(password: &str) -> Result<(), PasswordStrengthError> {
    if password.len() < 9 {
        return Err(PasswordStrengthError::TooShort);
    }

    if password.len() > 100 {
        return Err(PasswordStrengthError::TooLong);
    }

    // Check for at least one uppercase letter
    if !password.chars().any(char::is_uppercase) {
        return Err(PasswordStrengthError::MissingUppercase);
    }

    // Check for at least one lowercase letter
    if !password.chars().any(char::is_lowercase) {
        return Err(PasswordStrengthError::MissingLowercase);
    }

    // Check for at least one digit
    if !password.chars().any(char::is_numeric) {
        return Err(PasswordStrengthError::MissingDigit);
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_password() {
        let result = validate_password("ValidPassword123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_too_short_password() {
        let result = validate_password("Short");
        assert_eq!(result, Err(PasswordStrengthError::TooShort));
    }

    #[test]
    fn test_too_long_password() {
        let long_string = "aA3456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890";
        let result = validate_password(long_string);
        assert_eq!(result, Err(PasswordStrengthError::TooLong));
    }

    #[test]
    fn test_missing_uppercase() {
        let result = validate_password("lowercase123");
        assert_eq!(result, Err(PasswordStrengthError::MissingUppercase));
    }

    #[test]
    fn test_missing_lowercase() {
        let result = validate_password("UPPERCASE123");
        assert_eq!(result, Err(PasswordStrengthError::MissingLowercase));
    }

    #[test]
    fn test_missing_digit() {
        let result = validate_password("UpperCaseLowercase");
        assert_eq!(result, Err(PasswordStrengthError::MissingDigit));
    }

    #[test]
    fn test_hash_password_success() {
        let password = "test_passwordA1";
        let result = hash_password(password);
        assert!(result.is_ok());

        let hashed_password = result.unwrap();
        assert_ne!(hashed_password.len(), 0);
    }

    #[test]
    fn test_verify_password_valid() {
        let password = "correct_password";
        let yakman_password = YakManPassword {
            hash: hash_password(password).unwrap(),
            timestamp: 0,
        };

        assert!(verify_password(password, yakman_password).unwrap());
    }

    #[test]
    fn test_verify_password_invalid() {
        let password = "incorrect_password";
        let yakman_password = YakManPassword {
            hash: hash_password("correct_password").unwrap(),
            timestamp: 0,
        };

        assert!(!verify_password(password, yakman_password).unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash_format() {
        let password = "correct_password";
        let yakman_password = YakManPassword {
            hash: "invalid_hash_format".to_string(),
            timestamp: 0,
        };

        assert!(verify_password(password, yakman_password).is_err());
    }
}

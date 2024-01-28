use argon2::{
    password_hash::{rand_core::OsRng, Error, PasswordHasher, SaltString},
    Argon2,
};

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

pub fn is_valid_password(password: &str) -> bool {
    if password.len() < 9 || password.len() > 100 {
        return false;
    }

    // TODO: Added more passsword validation logic

    return true;
}

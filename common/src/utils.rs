use anyhow::{Ok, Result};
use bcrypt::{DEFAULT_COST, BcryptResult, hash, verify};

pub fn hash_string(input: &str) -> Result<String> {
    let result = hash(input, DEFAULT_COST)?;
    Ok(result)
}

pub fn verify_hash(input: &str, hashed: &str) -> Result<bool> {
    let result = verify(input, hashed)?;
    Ok(result)
}

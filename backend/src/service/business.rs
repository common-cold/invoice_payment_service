use anyhow::Result;
use rand::Rng;
use rand::distr::Alphanumeric;

pub fn generate_key() -> Result<String> {
    let key: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(15)
        .map(char::from)
        .collect();

    Ok(key)
}
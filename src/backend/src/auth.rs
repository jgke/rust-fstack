use frank_jwt::{Algorithm, ValidationOptions, encode, decode};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

const SECRET_KEY: &str = "foobar1234";

pub fn encrypt(mut payload: Value) -> Result<String, Box<dyn std::error::Error>> {
    let body = payload.as_object_mut().unwrap();

    let utc = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    body.insert("exp".to_string(), From::from(utc + 60 * 60 * 1));

    Ok(encode(json!({}), &SECRET_KEY, &payload, Algorithm::HS256).unwrap())
}

pub fn decrypt(s: &str) -> Result<(Value, Value), frank_jwt::Error> {
    decode(s, &SECRET_KEY, Algorithm::HS256, &ValidationOptions::default())
}

#[cfg(test)]
mod tests {
    #[test]
    fn encrypt_and_decrypt() -> Result<(), Box<dyn std::error::Error>> {
        let obj = json!({"user": 1234});
        let crypted = crate::auth::encrypt(obj)?;
        let decoded = crate::auth::decrypt(&crypted)?;
        assert_eq!(decoded.1.as_object().unwrap()["user"].as_i64().unwrap(), 1234);
        Ok(())
    }
}

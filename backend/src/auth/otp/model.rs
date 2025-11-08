use ring::hmac;
use serde::Deserialize;

use crate::database::Database;
use crate::error::AppError;
use crate::settings::Settings;

pub trait Model {
    async fn remember_otp(&self, email: &str, code: &str) -> Result<(), AppError>;
    async fn validate_otp(&self, email: &str, code: &str) -> Result<(), AppError>;
}

impl Model for Database {
    async fn remember_otp(&self, email: &str, code: &str) -> Result<(), AppError> {
        self.db
            .query(
                r#"
                DELETE otp WHERE expires_at <= time::now();
                LET $thing = type::thing('otp', $email);
                DELETE $thing;
                UPSERT $thing CONTENT {
                  code: $code,
                  expires_at: time::now() + duration::from::secs($ttl_secs),
                  created_at: time::now()
                };
                "#,
            )
            .bind(("email", email.to_owned()))
            .bind(("code", otp_hmac(email, code)))
            .bind(("ttl_secs", Settings::global().otp_ttl_seconds as i64))
            .await
            .map_err(AppError::database)?;
        Ok(())
    }

    async fn validate_otp(&self, email: &str, code: &str) -> Result<(), AppError> {
        #[derive(Deserialize)]
        struct Outcome {
            exists: i64,
            valid: i64,
        }

        let outcome = self.db
            .query(
                r#"
            DELETE otp WHERE expires_at <= time::now();
            LET $thing = type::thing('otp', $email);
            LET $exists = array::len(SELECT * FROM $thing);
            LET $valid = array::len(SELECT * FROM $thing WHERE code = $code AND expires_at > time::now());
            DELETE $thing WHERE code = $code AND expires_at > time::now() RETURN NONE;
            RETURN { exists: $exists, valid: $valid };
                "#
            )
            .bind(("email", email.to_owned()))
            .bind(("code", otp_hmac(email, code)))
            .await
            .map_err(AppError::database)?
            .take::<Option<Outcome>>(5).map_err(AppError::database)?
            .ok_or_else(|| AppError::invalid_request("no otp request for that email"))?;

        if outcome.valid > 0 {
            return Ok(());
        }
        if outcome.exists == 0 {
            return Err(AppError::invalid_request("no otp request for that email"));
        }
        Err(AppError::invalid_request("otp code is invalid"))
    }
}

fn otp_hmac(email: &str, code: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, Settings::global().otp_pepper.as_bytes());
    let data = format!("{}:{}", email, code);
    let tag = hmac::sign(&key, data.as_bytes());
    hex::encode(tag.as_ref())
}

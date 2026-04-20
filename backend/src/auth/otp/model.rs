use ring::hmac;
use serde::Deserialize;
use surrealdb::types::SurrealValue;

use crate::database::Database;
use crate::error::AppError;

pub trait Model {
    async fn remember_otp(
        &self,
        email: &str,
        code: &str,
        pepper: &str,
        ttl_seconds: u64,
    ) -> Result<(), AppError>;
    async fn validate_otp(
        &self,
        email: &str,
        code: &str,
        pepper: &str,
        max_attempts: u32,
    ) -> Result<(), AppError>;
}

impl Model for Database {
    async fn remember_otp(
        &self,
        email: &str,
        code: &str,
        pepper: &str,
        ttl_seconds: u64,
    ) -> Result<(), AppError> {
        self.db
            .query(
                r#"
                DELETE otp WHERE expires_at <= time::now();
                LET $thing = type::record('otp', $email);
                DELETE $thing;
                UPSERT $thing CONTENT {
                  code: $code,
                  expires_at: time::now() + duration::from_secs($ttl_secs),
                  created_at: time::now(),
                  failed_attempts: 0
                };
                "#,
            )
            .bind(("email", email.to_owned()))
            .bind(("code", otp_hmac(email, code, pepper)))
            .bind(("ttl_secs", ttl_seconds as i64))
            .await
            .map_err(|e| crate::log_and_convert!(AppError::database, "otp.remember.query", e))?;
        Ok(())
    }

    async fn validate_otp(
        &self,
        email: &str,
        code: &str,
        pepper: &str,
        max_attempts: u32,
    ) -> Result<(), AppError> {
        #[derive(Deserialize, SurrealValue)]
        struct Outcome {
            exists: i64,
            valid: i64,
            failed_attempts: i64,
        }

        // Query explanation:
        // 1. Clean up expired OTPs.
        // 2. On a valid code: delete the row and mark valid=1.
        // 3. On an invalid code with an active row: increment failed_attempts and return current count.
        let outcome = self
            .db
            .query(
                r#"
            DELETE otp WHERE expires_at <= time::now();
            LET $thing = type::record('otp', $email);
            LET $exists = array::len(SELECT * FROM $thing WHERE expires_at > time::now());
            LET $valid  = array::len(SELECT * FROM $thing WHERE code = $code AND expires_at > time::now());
            DELETE $thing WHERE code = $code AND expires_at > time::now() RETURN NONE;
            UPDATE $thing SET failed_attempts += 1 WHERE code != $code AND expires_at > time::now() RETURN NONE;
            LET $attempts = (SELECT VALUE failed_attempts FROM $thing WHERE expires_at > time::now())[0] ?? 0;
            RETURN { exists: $exists, valid: $valid, failed_attempts: $attempts };
                "#,
            )
            .bind(("email", email.to_owned()))
            .bind(("code", otp_hmac(email, code, pepper)))
            .await
            .map_err(|e| crate::log_and_convert!(AppError::database, "otp.validate.query", e))?
            .take::<Option<Outcome>>(7)
            .map_err(|e| crate::log_and_convert!(AppError::database, "otp.validate.take", e))?
            .ok_or_else(|| AppError::invalid_request("no otp request for that email"))?;

        if outcome.valid > 0 {
            return Ok(());
        }
        if outcome.exists == 0 {
            return Err(AppError::invalid_request("no otp request for that email"));
        }
        if outcome.failed_attempts >= max_attempts as i64 {
            // The row is still present but locked; delete it so further requests are also rejected.
            self.db
                .query("DELETE type::record('otp', $email)")
                .bind(("email", email.to_owned()))
                .await
                .map_err(|e| {
                    crate::log_and_convert!(AppError::database, "otp.lockout_delete.query", e)
                })?;
            return Err(AppError::too_many_requests(
                "too many incorrect otp attempts; request a new code",
            ));
        }
        Err(AppError::invalid_request("otp code is invalid"))
    }
}

fn otp_hmac(email: &str, code: &str, pepper: &str) -> String {
    let key = hmac::Key::new(hmac::HMAC_SHA256, pepper.as_bytes());
    let data = format!("{}:{}", email, code);
    let tag = hmac::sign(&key, data.as_bytes());
    hex::encode(tag.as_ref())
}

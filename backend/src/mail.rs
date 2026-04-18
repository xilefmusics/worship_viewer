use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::error::AppError;

#[derive(Clone)]
pub struct MailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl MailService {
    pub fn new(from: String, credentials: Credentials) -> Result<Self, AppError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.smtp_relay", e))?
            .credentials(credentials)
            .build();
        Ok(Self { transport, from })
    }

    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
        let message = Message::builder()
            .from(
                self.from
                    .parse()
                    .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.parse_from", e))?,
            )
            .to(to
                .parse()
                .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.parse_to", e))?)
            .subject(subject)
            .body(body.to_owned())
            .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.build_message", e))?;

        let response = self
            .transport
            .send(message)
            .await
            .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.transport_send", e))?;

        if !response.is_positive() {
            tracing::warn!(
                target = "mail.transport",
                ?response,
                "sending the mail was not positive"
            );
            return Err(AppError::mail("sending the mail was not positive"));
        }
        Ok(())
    }
}

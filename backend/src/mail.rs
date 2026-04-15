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
            .map_err(AppError::mail)?
            .credentials(credentials)
            .build();
        Ok(Self { transport, from })
    }

    pub async fn send(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
        let message = Message::builder()
            .from(self.from.parse().map_err(AppError::mail)?)
            .to(to.parse().map_err(AppError::mail)?)
            .subject(subject)
            .body(body.to_owned())
            .map_err(AppError::mail)?;

        let response = self.transport.send(message).await.map_err(AppError::mail)?;

        if !response.is_positive() {
            return Err(AppError::mail("sending the mail was not positive"));
        }
        Ok(())
    }
}

use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::instrument;

use crate::error::AppError;

#[derive(Clone)]
pub struct MailService {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: String,
}

impl MailService {
    #[instrument(level = "debug", err, skip(credentials), fields(from = %from))]
    pub fn new(from: String, credentials: Credentials) -> Result<Self, AppError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
            .map_err(|e| crate::log_and_convert!(AppError::mail, "mail.smtp_relay", e))?
            .credentials(credentials)
            .build();
        Ok(Self { transport, from })
    }

    #[instrument(
        level = "debug",
        err,
        skip(self, body),
        fields(
            to = tracing::field::display(to),
            subject = tracing::field::display(subject),
            transport_ok = tracing::field::Empty
        )
    )]
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
            tracing::Span::current().record("transport_ok", tracing::field::display(&false));
            tracing::warn!(
                target = "mail.transport",
                ?response,
                "sending the mail was not positive"
            );
            return Err(AppError::mail("sending the mail was not positive"));
        }
        tracing::Span::current().record("transport_ok", tracing::field::display(&true));
        Ok(())
    }
}

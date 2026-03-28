use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::error::AppError;
use crate::settings::Settings;

#[derive(Default)]
pub struct Mail<'a> {
    to: &'a str,
    subject: &'a str,
    body: &'a str,
}

impl<'a> Mail<'a> {
    pub fn to(mut self, to: &'a str) -> Self {
        self.to = to;
        self
    }
    pub fn subject(mut self, subject: &'a str) -> Self {
        self.subject = subject;
        self
    }
    pub fn body(mut self, body: &'a str) -> Self {
        self.body = body;
        self
    }

    pub fn send(self) -> Result<(), AppError> {
        let settings = Settings::global();

        let response = SmtpTransport::relay("smtp.gmail.com")
            .map_err(AppError::mail)?
            .credentials(Credentials::new(
                settings.gmail_from.to_owned(),
                settings.gmail_app_password.to_owned(),
            ))
            .build()
            .send(
                &Message::builder()
                    .from(
                        settings
                            .gmail_from
                            .parse()
                            .map_err(AppError::mail)?,
                    )
                    .to(self.to.parse().map_err(AppError::mail)?)
                    .subject(self.subject)
                    .body(self.body.to_owned())
                    .map_err(AppError::mail)?,
            )
            .map_err(AppError::mail)?;

        if !response.is_positive() {
            return Err(AppError::mail("sending the mail was not positive"));
        }

        Ok(())
    }
}

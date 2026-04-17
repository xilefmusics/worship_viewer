use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    message: String,
}

impl ValidationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

pub fn validate_resource_id(id: &str) -> Result<&str, ValidationError> {
    if id.is_empty() {
        return Err(ValidationError::new("id must not be empty"));
    }

    if id
        .chars()
        .any(|c| c.is_control() || c == '?' || c == '#' || c == '%' || c == ':')
    {
        return Err(ValidationError::new(
            "id contains forbidden characters (control, '?', '#', '%', ':')",
        ));
    }

    Ok(id)
}

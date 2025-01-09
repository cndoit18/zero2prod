use validator::ValidateEmail;
#[derive(Debug)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if ValidateEmail::validate_email(&s) {
            return Ok(Self(s));
        }
        Err(format!("{} is not a valid subscriber email.", s))
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for SubscriberEmail {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        let result = SubscriberEmail::parse(email);
        assert!(result.is_err());
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        let result = SubscriberEmail::parse(email);
        assert!(result.is_err());
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        let result = SubscriberEmail::parse(email);
        assert!(result.is_err());
    }
}

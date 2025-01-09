use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<Self, String> {
        let is_empty_or_whitespace = s.trim().is_empty();

        let is_too_long = s.graphemes(true).count() > 256;

        let forbidden_charachers = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_charachers.contains(&g));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_characters {
            return Err(format!("{} is not a vaild subscriber name.", s));
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsMut<str> for SubscriberName {
    fn as_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_256_grapheme_long_name_is_vaild() {
        let name = "ά".repeat(256);
        let result = SubscriberName::parse(name);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "ά".repeat(257);
        let result = SubscriberName::parse(name);
        assert!(result.is_err());
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        let result = SubscriberName::parse(name);
        assert!(result.is_err());
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        let result = SubscriberName::parse(name);
        assert!(result.is_err());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            let result = SubscriberName::parse(name);
            assert!(result.is_err());
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Ursula Le Guin".to_string();
        let result = SubscriberName::parse(name);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}

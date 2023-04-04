use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

const MAX_NAME_LENGTH: usize = 256;
const FORBIDDEN_CHARS: [char; 9] = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

impl SubscriberName {
    pub fn parse(value: String) -> Result<SubscriberName, String> {
        if value.trim().is_empty() {
            return Err("name is empty".to_string());
        }

        if value.graphemes(true).count() > MAX_NAME_LENGTH {
            return Err("name too large".to_string());
        }

        if value.chars().any(|g| FORBIDDEN_CHARS.contains(&g)) {
            return Err("contains illegal characters".to_string());
        }

        Ok(Self(value))
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};

    use crate::domain::SubscriberName;

    use super::FORBIDDEN_CHARS;

    #[test]
    fn name_of_exactly_256_graphemes_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn name_longer_than_256_graphemes_is_invalid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn whitespace_only_string_is_invalid() {
        let name = " ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_is_invalid() {
        assert_err!(SubscriberName::parse("".to_string()));
    }

    #[test]
    fn names_containing_forbidden_characters_are_invalid() {
        for name in FORBIDDEN_CHARS {
            assert_err!(SubscriberName::parse(name.to_string()));
        }
    }

    #[test]
    fn a_normal_name_is_valid() {
        assert_ok!(SubscriberName::parse("John Doe".to_string()));
    }
}

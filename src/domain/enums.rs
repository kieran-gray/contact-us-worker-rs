use serde::{Deserialize, Serialize};
use strum_macros::EnumString;

#[derive(Debug, Clone, Deserialize, Serialize, EnumString, PartialEq)]
pub enum ContactMessageCategory {
    #[strum(serialize = "ERROR", serialize = "error")]
    ERROR,
    #[strum(serialize = "IDEA", serialize = "idea")]
    IDEA,
    #[strum(serialize = "TESTIMONIAL", serialize = "testimonial")]
    TESTIMONIAL,
    #[strum(serialize = "OTHER", serialize = "other")]
    OTHER,
}

impl std::fmt::Display for ContactMessageCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContactMessageCategory::ERROR => write!(f, "ERROR"),
            ContactMessageCategory::IDEA => write!(f, "IDEA"),
            ContactMessageCategory::TESTIMONIAL => write!(f, "TESTIMONIAL"),
            ContactMessageCategory::OTHER => write!(f, "OTHER"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ContactMessageCategory;

    #[test]
    fn display_formats_correctly() {
        assert_eq!(ContactMessageCategory::ERROR.to_string(), "ERROR");
        assert_eq!(ContactMessageCategory::IDEA.to_string(), "IDEA");
        assert_eq!(
            ContactMessageCategory::TESTIMONIAL.to_string(),
            "TESTIMONIAL"
        );
        assert_eq!(ContactMessageCategory::OTHER.to_string(), "OTHER");
    }
}

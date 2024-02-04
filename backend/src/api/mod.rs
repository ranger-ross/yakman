pub mod admin;
pub mod auth;
pub mod configs;
pub mod data;
pub mod instances;
pub mod labels;
pub mod oauth;
pub mod projects;
pub mod revisions;
pub mod yakman;

fn is_alphanumeric_kebab_case(s: &str) -> bool {
    s.chars().all(|c| c == '-' || c.is_alphanumeric())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_return_true_for_alphanumeric_kebab_case_strings() {
        assert!(is_alphanumeric_kebab_case("hello"));
        assert!(is_alphanumeric_kebab_case("hello-something"));
        assert!(is_alphanumeric_kebab_case("some"));
        assert!(is_alphanumeric_kebab_case("another-with-multiple-hyphens"));
        assert!(is_alphanumeric_kebab_case("a"));
        assert!(is_alphanumeric_kebab_case("a-number5"));
        assert!(is_alphanumeric_kebab_case("a-4"));
        assert!(is_alphanumeric_kebab_case("a43"));
        assert!(is_alphanumeric_kebab_case("100"));
    }

    #[test]
    fn should_return_false_for_non_alphanumeric_kebab_case_strings() {
        assert!(!is_alphanumeric_kebab_case(" hello"));
        assert!(!is_alphanumeric_kebab_case("hello "));
        assert!(!is_alphanumeric_kebab_case(" "));
        assert!(!is_alphanumeric_kebab_case(" "));
        assert!(!is_alphanumeric_kebab_case("!"));
        assert!(!is_alphanumeric_kebab_case("!"));
        assert!(!is_alphanumeric_kebab_case("%"));
        assert!(!is_alphanumeric_kebab_case("hello world"));
        assert!(!is_alphanumeric_kebab_case("hello%20world"));
        assert!(!is_alphanumeric_kebab_case("%20"));
    }
}

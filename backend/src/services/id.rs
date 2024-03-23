use uuid::Uuid;

pub fn generate_project_id() -> String {
    return format!("p{}", short_sha(&Uuid::new_v4().to_string()));
}

pub fn generate_config_id() -> String {
    return format!("c{}", short_sha(&Uuid::new_v4().to_string()));
}

pub fn generate_user_id() -> String {
    return format!("u{}", short_sha(&Uuid::new_v4().to_string()));
}

pub fn generate_instance_id() -> String {
    return format!("i{}", short_sha(&Uuid::new_v4().to_string()));
}

pub fn generate_revision_id() -> String {
    return format!("r{}", short_sha(&Uuid::new_v4().to_string()));
}

/// Returns a 12 character string representation of a SHA256
pub fn short_sha(input: &str) -> String {
    let sha: String = sha256::digest(input);
    return sha[0..12].to_string();
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_short_sha() {
        let result = short_sha("hello world");
        assert_eq!(result, "b94d27b9934d");

        let result = short_sha("foo");
        assert_eq!(result, "2c26b46b68ff");

        let result = short_sha("bar");
        assert_eq!(result, "fcde2b2edba5");

        let result = short_sha("ade10004-41df-4bf6-88b9-d768afab674f");
        assert_eq!(result, "8146205a8d27");
    }

    #[test]
    fn test_generate_instance_id() {
        for _i in 0..10 {
            let result = generate_instance_id();
            assert_eq!(13, result.len());
            assert!(result.starts_with('i'));
        }
    }

    #[test]
    fn test_generate_revision_id() {
        for _i in 0..10 {
            let result = generate_revision_id();
            assert_eq!(13, result.len());
            assert!(result.starts_with('r'));
        }
    }
}

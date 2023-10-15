use actix_web::dev::ServiceRequest;

pub fn extract_access_token(req: &ServiceRequest) -> Option<String> {
    if let Some(token_header) = req.headers().get("Authorization") {
        if let Ok(token) = token_header.to_str() {
            let parts: Vec<&str> = token.split(" ").collect();

            if parts.len() != 2 {
                return None;
            }

            if parts[0].to_lowercase() != "bearer" {
                return None;
            }

            return Some(parts[1].to_string());
        }
    }
    None
}

#[cfg(test)]
mod test {
    use actix_web::test::TestRequest;

    use super::*;

    #[test]
    fn extract_access_token_valid() {
        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer 123"))
            .to_srv_request();

        assert_eq!(Some("123".to_string()), extract_access_token(&req));

        let req = TestRequest::default()
            .insert_header(("Authorization", "bearer 123"))
            .to_srv_request();

        assert_eq!(Some("123".to_string()), extract_access_token(&req));
    }

    #[test]
    fn extract_access_token_invalid() {
        let req = TestRequest::default().to_srv_request();
        assert_eq!(None, extract_access_token(&req));

        let req = TestRequest::default()
            .insert_header(("Authorization", "123"))
            .to_srv_request();
        assert_eq!(None, extract_access_token(&req));

        let req = TestRequest::default()
            .insert_header(("Authorization", "Bearer"))
            .to_srv_request();
        assert_eq!(None, extract_access_token(&req));

        let req = TestRequest::default()
            .insert_header(("Authorization", "Token 123"))
            .to_srv_request();
        assert_eq!(None, extract_access_token(&req));
    }
}

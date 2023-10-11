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


// TODO: Write unit tets for this
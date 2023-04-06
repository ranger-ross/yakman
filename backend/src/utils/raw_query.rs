use rocket::{
    request::{FromRequest, Outcome},
    Request,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RawQuery {
    pub params: HashMap<String, String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RawQuery {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let uri = request.uri().to_string();
        let mut m: HashMap<String, String> = HashMap::new();

        if uri.contains("?") {
            let index = uri.find("?").unwrap();
            let query: String = uri.chars().skip(index + 1).collect();

            for param in query.split("&") {
                let mut parts = param.split("=");
                let key = parts.next().unwrap();
                let value = parts.next().unwrap();
                m.insert(key.to_string(), value.to_string());
            }
        }

        return Outcome::Success(RawQuery { params: m });
    }
}

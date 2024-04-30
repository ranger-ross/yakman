use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::YakManRole;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CreateYakManUserPayload {
    pub email: String,
    pub role: Option<YakManRole>,
}

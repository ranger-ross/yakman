use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct InstancePayload {
    pub instance: String,
}

use std::collections::HashMap;

use super::YakManRole;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct GetUserRolesResponse {
    pub global_roles: Vec<YakManRole>,
    pub roles: HashMap<String, YakManRole>,
}

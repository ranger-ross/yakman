extern crate dotenv;

use std::collections::HashSet;
use std::sync::Arc;

use super::token::extract_access_token;
use super::YakManPrinciple;
use crate::auth::token::{TokenService, YakManTokenService};
use crate::model::{YakManRole, YakManUserProjectRole};
use crate::services::StorageService;
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, web, Error};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum YakManRoleBinding {
    GlobalRoleBinding(YakManRole),
    ProjectRoleBinding(YakManUserProjectRole),
}

impl YakManRoleBinding {
    pub fn has_any_role(
        roles_to_match: Vec<YakManRole>,
        project_uuid: &str,
        roles: &HashSet<YakManRoleBinding>,
    ) -> bool {
        for role in roles {
            match role {
                YakManRoleBinding::GlobalRoleBinding(r) => {
                    if r == &YakManRole::Admin {
                        return true;
                    }

                    if roles_to_match.contains(&r) {
                        return true;
                    }
                }
                YakManRoleBinding::ProjectRoleBinding(r) => {
                    if r.project_uuid == project_uuid {
                        if r.role == YakManRole::Admin {
                            return true;
                        }

                        if roles_to_match.contains(&r.role) {
                            return true;
                        }
                    }
                }
            }
        }

        return false;
    }

    #[allow(dead_code)]
    pub fn has_role(
        role_to_match: YakManRole,
        project_uuid: &str,
        roles: &HashSet<YakManRoleBinding>,
    ) -> bool {
        return YakManRoleBinding::has_any_role(vec![role_to_match], project_uuid, roles);
    }

    pub fn has_global_role(role_to_match: YakManRole, roles: &HashSet<YakManRoleBinding>) -> bool {
        return YakManRoleBinding::has_any_global_role(vec![role_to_match], roles);
    }

    pub fn has_any_global_role(
        roles_to_match: Vec<YakManRole>,
        roles: &HashSet<YakManRoleBinding>,
    ) -> bool {
        for role in roles {
            match role {
                YakManRoleBinding::GlobalRoleBinding(r) => {
                    if r == &YakManRole::Admin {
                        return true;
                    }

                    if roles_to_match.contains(&r) {
                        return true;
                    }
                }
                _ => {}
            }
        }

        return false;
    }
}

pub async fn extract_roles(req: &ServiceRequest) -> Result<HashSet<YakManRoleBinding>, Error> {
    let mut role_bindings: HashSet<YakManRoleBinding> = HashSet::new();

    let token_service = req
        .app_data::<web::Data<Arc<YakManTokenService>>>()
        .unwrap();

    let token: Option<String> = extract_access_token(req);

    let token = match token {
        Some(token) => token,
        None => return Ok(HashSet::new()),
    };

    if token_service.is_api_key(&token) {
        return match req.extensions().get::<YakManPrinciple>() {
            Some(principle) => {
                let key_id = match &principle.user_uuid {
                    Some(key_id) => key_id,
                    None => return Ok(HashSet::new()),
                };

                let storage_service = req
                    .app_data::<web::Data<Arc<dyn StorageService>>>()
                    .unwrap();

                if let Some(api_key) = storage_service.get_api_key_by_id(&key_id).await.unwrap() {
                    let mut api_key_roles = HashSet::new();
                    api_key_roles.insert(YakManRoleBinding::ProjectRoleBinding(
                        YakManUserProjectRole {
                            project_uuid: api_key.project_uuid,
                            role: api_key.role,
                        },
                    ));

                    Ok(api_key_roles)
                } else {
                    Ok(HashSet::new())
                }
            }
            None => Ok(HashSet::new()),
        };
    }

    match token_service.validate_access_token(&token) {
        Ok(claims) => {
            let uuid = claims.uuid;

            let storage_service = req
                .app_data::<web::Data<Arc<dyn StorageService>>>()
                .unwrap();

            if let Some(details) = storage_service.get_user_details(&uuid).await? {
                let global_roles: Vec<YakManRoleBinding> = details
                    .global_roles
                    .iter()
                    .map(|p| YakManRoleBinding::GlobalRoleBinding(p.clone()))
                    .collect();

                role_bindings.extend(global_roles);

                let project_role_bindings: Vec<YakManRoleBinding> = details
                    .roles
                    .into_iter()
                    .map(|p| YakManRoleBinding::ProjectRoleBinding(p))
                    .collect();

                role_bindings.extend(project_role_bindings);
            } else {
                log::info!("user details not found");
            }
        }
        Err(e) => {
            log::info!("token invalid {e:?}");
        }
    }

    return Ok(role_bindings);
}

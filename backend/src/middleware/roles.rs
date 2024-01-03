extern crate dotenv;

use super::token::extract_access_token;
use super::YakManPrinciple;
use crate::model::{YakManRole, YakManUserProjectRole};
use crate::StateManager;
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, web, Error};

#[derive(Debug, PartialEq, Clone)]
pub enum YakManRoleBinding {
    GlobalRoleBinding(YakManRole),
    ProjectRoleBinding(YakManUserProjectRole),
}

impl YakManRoleBinding {
    pub fn has_any_role(
        roles_to_match: Vec<YakManRole>,
        project_uuid: &str,
        roles: &Vec<YakManRoleBinding>,
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
        roles: &Vec<YakManRoleBinding>,
    ) -> bool {
        return YakManRoleBinding::has_any_role(vec![role_to_match], project_uuid, roles);
    }

    pub fn has_global_role(role_to_match: YakManRole, roles: &Vec<YakManRoleBinding>) -> bool {
        return YakManRoleBinding::has_any_global_role(vec![role_to_match], roles);
    }

    pub fn has_any_global_role(
        roles_to_match: Vec<YakManRole>,
        roles: &Vec<YakManRoleBinding>,
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

pub async fn extract_roles(req: &ServiceRequest) -> Result<Vec<YakManRoleBinding>, Error> {
    let mut role_bindings: Vec<YakManRoleBinding> = vec![];

    let state = req.app_data::<web::Data<StateManager>>().unwrap();
    let token_service = state.get_token_service();
    let token: Option<String> = extract_access_token(req);

    let token = match token {
        Some(token) => token,
        None => return Ok(vec![]),
    };

    if token_service.is_api_key(&token) {
        return match req.extensions().get::<YakManPrinciple>() {
            Some(principle) => {
                let key_id = match &principle.api_key_id {
                    Some(key_id) => key_id,
                    None => return Ok(vec![]),
                };

                if let Some(api_key) = state
                    .get_service()
                    .get_api_key_by_id(&key_id)
                    .await
                    .unwrap()
                {
                    Ok(vec![YakManRoleBinding::ProjectRoleBinding(
                        YakManUserProjectRole {
                            project_uuid: api_key.project_uuid,
                            role: api_key.role,
                        },
                    )])
                } else {
                    Ok(vec![])
                }
            }
            None => Ok(vec![]),
        };
    }

    match token_service.validate_access_token(&token) {
        Ok(claims) => {
            let uuid = claims.uuid;

            if let Some(details) = state.get_service().get_user_details(&uuid).await? {
                let mut global_roles: Vec<YakManRoleBinding> = details
                    .global_roles
                    .iter()
                    .map(|p| YakManRoleBinding::GlobalRoleBinding(p.clone()))
                    .collect();

                role_bindings.append(&mut global_roles);

                let mut project_role_bindings: Vec<YakManRoleBinding> = details
                    .roles
                    .into_iter()
                    .map(|p| YakManRoleBinding::ProjectRoleBinding(p))
                    .collect();

                role_bindings.append(&mut project_role_bindings);
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

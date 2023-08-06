extern crate dotenv;

use crate::auth::oauth_service::OAUTH_ACCESS_TOKEN_COOKIE_NAME;
use crate::StateManager;
use actix_web::{dev::ServiceRequest, web, Error};
use log::info;
use crate::model::{YakManRole, YakManUserProjectRole};

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
    let cookies = req.cookies().unwrap();
    let token = cookies
        .iter()
        .find(|c| c.name() == OAUTH_ACCESS_TOKEN_COOKIE_NAME);

    if token.is_none() {
        return Ok(vec![]);
    }

    match state
        .get_token_service()
        .validate_access_token(token.unwrap().value())
    {
        Ok(claims) => {
            let uuid = claims.uuid;

            let details = state
                .get_service()
                .get_user_details(&uuid)
                .await
                .unwrap()
                .unwrap(); // TODO: handle these unwraps

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
        }
        Err(e) => {
            info!("token invalid {e:?}");
        }
    }

    return Ok(role_bindings);
}

extern crate dotenv;

use std::collections::HashSet;
use std::sync::Arc;

use super::token::extract_access_token;
use super::YakManPrinciple;
use crate::auth::token::{TokenService, YakManTokenService};
use crate::model::{YakManProjectRole, YakManRole};
use crate::services::StorageService;
use actix_web::HttpMessage;
use actix_web::{dev::ServiceRequest, web, Error};
use futures_util::future::join_all;
use futures_util::TryFutureExt;

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum YakManRoleBinding {
    GlobalRoleBinding(YakManRole),
    ProjectRoleBinding(YakManProjectRole),
}

impl YakManRoleBinding {
    pub fn has_any_role(
        roles_to_match: Vec<YakManRole>,
        project_id: &str,
        roles: &HashSet<YakManRoleBinding>,
    ) -> bool {
        for role in roles {
            match role {
                YakManRoleBinding::GlobalRoleBinding(r) => {
                    if r == &YakManRole::Admin {
                        return true;
                    }

                    if roles_to_match.contains(r) {
                        return true;
                    }
                }
                YakManRoleBinding::ProjectRoleBinding(r) => {
                    if r.project_id == project_id {
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
        project_id: &str,
        roles: &HashSet<YakManRoleBinding>,
    ) -> bool {
        return YakManRoleBinding::has_any_role(vec![role_to_match], project_id, roles);
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

                    if roles_to_match.contains(r) {
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
                let key_id = match &principle.user_id {
                    Some(key_id) => key_id,
                    None => return Ok(HashSet::new()),
                };

                let storage_service = req
                    .app_data::<web::Data<Arc<dyn StorageService>>>()
                    .unwrap();

                if let Some(api_key) = storage_service.get_api_key_by_id(key_id).await.unwrap() {
                    let mut api_key_roles = HashSet::new();
                    api_key_roles.insert(YakManRoleBinding::ProjectRoleBinding(
                        YakManProjectRole {
                            project_id: api_key.project_id,
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
            let user_id = claims.user_id;

            let storage_service = req
                .app_data::<web::Data<Arc<dyn StorageService>>>()
                .unwrap();

            if let Some(details) = storage_service.get_user_details(&user_id).await? {
                let global_roles: Vec<YakManRoleBinding> = details
                    .global_roles
                    .iter()
                    .map(|p| YakManRoleBinding::GlobalRoleBinding(p.clone()))
                    .collect();

                role_bindings.extend(global_roles);

                let project_role_bindings: Vec<YakManRoleBinding> = details
                    .roles
                    .into_iter()
                    .map(YakManRoleBinding::ProjectRoleBinding)
                    .collect();

                role_bindings.extend(project_role_bindings);

                // If the user already has global admin role, we can skip loading team roles
                if !role_bindings.contains(&YakManRoleBinding::GlobalRoleBinding(YakManRole::Admin))
                {
                    // Load team roles
                    let futures: Vec<_> = details
                        .team_ids
                        .iter()
                        .map(|team_id| {
                            storage_service
                                .get_team_details(team_id)
                                .map_ok(move |inner| {
                                    inner.ok_or(format!("Team with ID not found {team_id}"))
                                })
                        })
                        .collect();

                    for result in join_all(futures).await {
                        let team_details = match result {
                            Ok(Ok(team_details)) => team_details,
                            Ok(Err(err)) => {
                                log::warn!("Could not load team to get roles {err:?}");
                                continue;
                            }
                            Err(err) => {
                                log::warn!("Could not load team to get roles {err:?}");
                                continue;
                            }
                        };
                        let global_roles: Vec<YakManRoleBinding> = team_details
                            .global_roles
                            .iter()
                            .map(|p| YakManRoleBinding::GlobalRoleBinding(p.clone()))
                            .collect();
                        role_bindings.extend(global_roles);

                        let project_role_bindings: Vec<YakManRoleBinding> = team_details
                            .roles
                            .into_iter()
                            .map(YakManRoleBinding::ProjectRoleBinding)
                            .collect();

                        role_bindings.extend(project_role_bindings);
                    }
                }
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

pub mod admin;
pub mod auth;
pub mod configs;
pub mod data;
pub mod instances;
pub mod labels;
pub mod oauth;
pub mod projects;
pub mod revisions;
pub mod yakman;

use self::{
    admin::{CreateApiKeyRequest, CreateApiKeyResponse},
    auth::{
        CreatePasswordResetLink, LoginRequest, PasswordResetPayload, ValidatePasswordResetLink,
    },
    oauth::{
        GetUserInfoResponse, OAuthExchangePayload, OAuthInitPayload, OAuthInitResponse,
        OAuthRefreshTokenPayload,
    },
    revisions::ReviewResult,
    yakman::YakManSettingsResponse,
};
use crate::model::{
    request::{CreateConfigPayload, CreateProjectPayload, DeleteConfigPayload},
    response::{InstancePayload, RevisionPayload},
    ConfigInstance, ConfigInstanceChange, ConfigInstanceRevision, LabelType, RevisionReviewState,
    YakManConfig, YakManLabel, YakManProject, YakManPublicPasswordResetLink, YakManRole,
    YakManUser,
};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    App,
};
use oauth2::PkceCodeVerifier;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        yakman::yakman_settings,
        auth::login,
        auth::reset_password,
        auth::create_password_reset_link,
        auth::validate_password_reset_link,
        oauth::oauth_init,
        oauth::oauth_exchange,
        oauth::oauth_refresh,
        oauth::get_user_info,
        projects::get_projects,
        projects::create_project,
        configs::get_configs,
        configs::create_config,
        configs::delete_config,
        labels::get_labels,
        labels::create_label,
        instances::get_instances_by_config_name,
        instances::get_instance,
        instances::create_new_instance,
        instances::update_new_instance,
        instances::delete_instance,
        data::get_instance_data,
        data::get_revision_data,
        revisions::get_instance_revisions,
        revisions::review_pending_instance_revision,
        revisions::apply_instance_revision,
        revisions::rollback_instance_revision,
        admin::get_yakman_users,
        admin::create_yakman_user,
        admin::get_api_keys,
        admin::create_api_key,
        admin::delete_api_key,
    ),
    components(
        schemas(
            YakManConfig, LabelType, YakManLabel, ConfigInstance, ConfigInstanceRevision, ConfigInstanceChange,
            YakManProject, YakManRole, YakManUser, CreateConfigPayload, CreateProjectPayload, GetUserInfoResponse,
            OAuthInitPayload, OAuthExchangePayload, OAuthInitResponse, RevisionPayload, OAuthRefreshTokenPayload,
            CreatePasswordResetLink, LoginRequest, PasswordResetPayload, YakManPublicPasswordResetLink, ValidatePasswordResetLink,
            DeleteConfigPayload, RevisionReviewState, ReviewResult, InstancePayload, YakManSettingsResponse, CreateApiKeyRequest,
            CreateApiKeyResponse
        )
    ),
    tags(
        (name = "oauth", description = "OAuth endpoints"),
        (name = "auth", description = "Authentication endpoints (non-oauth)"),
        (name = "projects", description = "Project management endpoints"),
        (name = "configs", description = "Config management endpoints"),
        (name = "labels", description = "Label management endpoints"),
        (name = "instances", description = "Config Instance management endpoints"),
        (name = "data", description = "Config data fetching endpoints"),
        (name = "revisions", description = "Config Instance Revision management endpoints"),
        (name = "yakman", description = "YakMan settings"),
        (name = "admin", description = "YakMan management (admins only) endpoints"),
    )
)]
pub struct YakManApiDoc;

fn t(x: CreateApiKeyRequest) {}

pub fn register_routes<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
{
    app
        // YakMan
        .service(yakman::yakman_settings)
        // Auth
        .service(auth::login)
        .service(auth::reset_password)
        .service(auth::create_password_reset_link)
        .service(auth::validate_password_reset_link)
        // OAuth
        .service(oauth::oauth_init)
        .service(oauth::oauth_exchange)
        .service(oauth::oauth_refresh)
        .service(oauth::get_user_info)
        // Projects
        .service(projects::get_projects)
        .service(projects::create_project)
        // Admin
        .service(admin::get_yakman_users)
        .service(admin::create_yakman_user)
        .service(admin::get_api_keys)
        .service(admin::create_api_key)
        .service(admin::delete_api_key)
        // Configs
        .service(configs::get_configs)
        .service(configs::create_config)
        .service(configs::delete_config)
        // Labels
        .service(labels::get_labels)
        .service(labels::create_label)
        // Instances
        .service(instances::get_instances_by_config_name)
        .service(instances::get_instance)
        .service(instances::create_new_instance)
        .service(instances::update_new_instance)
        .service(instances::delete_instance)
        // Data
        .service(data::get_instance_data)
        .service(data::get_revision_data)
        // Revisions
        .service(revisions::get_instance_revisions)
        .service(revisions::review_pending_instance_revision)
        .service(revisions::apply_instance_revision)
        .service(revisions::rollback_instance_revision)
}

fn is_alphanumeric_kebab_case(s: &str) -> bool {
    s.chars().all(|c| c == '-' || c.is_alphanumeric())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_return_true_for_alphanumeric_kebab_case_strings() {
        assert!(is_alphanumeric_kebab_case("hello"));
        assert!(is_alphanumeric_kebab_case("hello-something"));
        assert!(is_alphanumeric_kebab_case("some"));
        assert!(is_alphanumeric_kebab_case("another-with-multiple-hyphens"));
        assert!(is_alphanumeric_kebab_case("a"));
        assert!(is_alphanumeric_kebab_case("a-number5"));
        assert!(is_alphanumeric_kebab_case("a-4"));
        assert!(is_alphanumeric_kebab_case("a43"));
        assert!(is_alphanumeric_kebab_case("100"));
    }

    #[test]
    fn should_return_false_for_non_alphanumeric_kebab_case_strings() {
        assert!(!is_alphanumeric_kebab_case(" hello"));
        assert!(!is_alphanumeric_kebab_case("hello "));
        assert!(!is_alphanumeric_kebab_case(" "));
        assert!(!is_alphanumeric_kebab_case(" "));
        assert!(!is_alphanumeric_kebab_case("!"));
        assert!(!is_alphanumeric_kebab_case("!"));
        assert!(!is_alphanumeric_kebab_case("%"));
        assert!(!is_alphanumeric_kebab_case("hello world"));
        assert!(!is_alphanumeric_kebab_case("hello%20world"));
        assert!(!is_alphanumeric_kebab_case("%20"));
    }
}

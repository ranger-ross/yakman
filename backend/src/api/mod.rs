pub mod api_keys;
pub mod auth;
pub mod configs;
pub mod data;
pub mod instances;
pub mod labels;
pub mod lifecycle;
pub mod projects;
pub mod revisions;
pub mod teams;
pub mod users;

use self::{
    api_keys::{CreateApiKeyRequest, CreateApiKeyResponse},
    auth::{
        CreatePasswordResetLink, LoginRequest, OAuthExchangePayload, OAuthInitPayload,
        OAuthInitResponse, OAuthRefreshTokenPayload, PasswordResetPayload,
        ValidatePasswordResetLink,
    },
    lifecycle::{YakManHealthResponse, YakManSettingsResponse},
    revisions::ReviewResult,
    teams::CreateTeamResponse,
    users::GetUserInfoResponse,
};
use crate::model::{
    request::{
        CreateConfigPayload, CreateProjectPayload, CreateTeamPayload, DeleteConfigPayload,
        ProjectNotificationType, UpdateProjectPayload, UpdateTeamPayload,
    },
    response::{InstancePayload, RevisionPayload},
    ConfigInstance, ConfigInstanceEvent, ConfigInstanceEventData, ConfigInstanceRevision,
    LabelType, NotificationSetting, NotificationSettingEvents, ProjectNotificationSettings,
    RevisionReviewState, YakManConfig, YakManLabel, YakManProject, YakManProjectDetails,
    YakManPublicPasswordResetLink, YakManRole, YakManTeam, YakManTeamDetails, YakManUser,
};
use actix_web::{
    dev::{ServiceFactory, ServiceRequest},
    App,
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        lifecycle::health,
        lifecycle::yakman_settings,
        auth::login,
        auth::reset_password,
        auth::create_password_reset_link,
        auth::validate_password_reset_link,
        auth::oauth_init,
        auth::oauth_exchange,
        auth::oauth_refresh,
        projects::get_projects,
        projects::get_project,
        projects::create_project,
        projects::update_project,
        projects::delete_project,
        configs::get_configs,
        configs::create_config,
        configs::delete_config,
        labels::get_labels,
        labels::create_label,
        instances::get_instances_by_config_id,
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
        users::get_yakman_users,
        users::create_yakman_user,
        users::get_user_info,
        teams::get_teams,
        teams::get_team,
        teams::create_team,
        teams::update_team,
        teams::delete_team,
        api_keys::get_api_keys,
        api_keys::create_api_key,
        api_keys::delete_api_key,
    ),
    components(
        schemas(
            YakManConfig, LabelType, YakManLabel, ConfigInstance, ConfigInstanceRevision, ConfigInstanceEvent,
            YakManProject, YakManRole, YakManUser, CreateConfigPayload, CreateProjectPayload, GetUserInfoResponse,
            OAuthInitPayload, OAuthExchangePayload, OAuthInitResponse, RevisionPayload, OAuthRefreshTokenPayload,
            CreatePasswordResetLink, LoginRequest, PasswordResetPayload, YakManPublicPasswordResetLink, ValidatePasswordResetLink,
            DeleteConfigPayload, RevisionReviewState, ReviewResult, InstancePayload, YakManSettingsResponse, CreateApiKeyRequest,
            CreateApiKeyResponse, YakManHealthResponse, ConfigInstanceEventData, ProjectNotificationType, ProjectNotificationSettings,
            YakManProjectDetails, NotificationSettingEvents, NotificationSetting, UpdateProjectPayload, UpdateTeamPayload,
            CreateTeamPayload, YakManTeam, CreateTeamResponse, YakManTeamDetails
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "projects", description = "Project management endpoints"),
        (name = "configs", description = "Config management endpoints"),
        (name = "labels", description = "Label management endpoints"),
        (name = "instances", description = "Config Instance management endpoints"),
        (name = "data", description = "Config data fetching endpoints"),
        (name = "revisions", description = "Config Instance Revision management endpoints"),
        (name = "users", description = "YakMan user management endpoints"),
        (name = "teams", description = "YakMan team management endpoints"),
        (name = "lifecycle", description = "Application lifecycle endpoints"),
        (name = "api_keys", description = "API Key management endpoints"),
    )
)]
pub struct YakManApiDoc;

pub fn register_routes<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = actix_web::Error, InitError = ()>,
{
    app
        // Lifecycle
        .service(lifecycle::health)
        .service(lifecycle::yakman_settings)
        // Auth
        .service(auth::login)
        .service(auth::reset_password)
        .service(auth::create_password_reset_link)
        .service(auth::validate_password_reset_link)
        .service(auth::oauth_init)
        .service(auth::oauth_exchange)
        .service(auth::oauth_refresh)
        // Projects
        .service(projects::get_projects)
        .service(projects::get_project)
        .service(projects::create_project)
        .service(projects::update_project)
        .service(projects::delete_project)
        // Users
        .service(users::get_yakman_users)
        .service(users::create_yakman_user)
        .service(users::get_user_info)
        // Teams
        .service(teams::get_teams)
        .service(teams::get_team)
        .service(teams::update_team)
        .service(teams::create_team)
        .service(teams::delete_team)
        // Api Keys
        .service(api_keys::get_api_keys)
        .service(api_keys::create_api_key)
        .service(api_keys::delete_api_key)
        // Configs
        .service(configs::get_configs)
        .service(configs::create_config)
        .service(configs::delete_config)
        // Labels
        .service(labels::get_labels)
        .service(labels::create_label)
        // Instances
        .service(instances::get_instances_by_config_id)
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
    use std::collections::BTreeMap;

    use utoipa::openapi::{RefOr, Schema};

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

    #[test]
    fn require_all_schemas_to_be_added_to_openapi_spec() {
        let openapi = YakManApiDoc::openapi();

        let binding = openapi.components.unwrap_or_default();
        let schemas = binding.schemas;
        let paths = openapi.paths.paths;

        for (path, path_item) in paths {
            for (_path_type, operation) in path_item.operations {
                if let Some(request_body) = operation.request_body {
                    for (_content_type, content) in request_body.content {
                        if let RefOr::Ref(r) = content.schema {
                            verify_ref_location(&r.ref_location, &path, &schemas);
                        }
                    }
                }

                for (_status, response) in operation.responses.responses {
                    if let RefOr::T(res) = response {
                        for (_content_type, content) in res.content {
                            if let RefOr::Ref(r) = content.schema {
                                verify_ref_location(&r.ref_location, &path, &schemas);
                            }
                        }
                    }
                }
            }
        }
    }

    fn fail_missing_schema(schema_name: &str, path: &str) {
        panic!("Could not find `{schema_name}` in #/components/schemas for `{path}`. Did you forget to add it to #[openapi(...)] macro?");
    }

    fn verify_ref_location(
        ref_location: &str,
        path: &str,
        schemas: &BTreeMap<String, RefOr<Schema>>,
    ) {
        let prefix = "#/components/schemas/";

        if ref_location.starts_with(prefix) {
            let schema_name = &ref_location[prefix.len()..];
            if !schemas.contains_key(schema_name) {
                fail_missing_schema(schema_name, &path);
            }
        } else {
            eprintln!(
                "[WARN] Unexpected location {ref_location} (not part of #/components/schemas)"
            );
        }
    }
}

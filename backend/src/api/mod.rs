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
pub mod validation;

use actix_web::web;
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
        labels::update_label,
        labels::delete_label,
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

pub fn register_routes(config: &mut web::ServiceConfig) {
    config
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
        .service(labels::update_label)
        .service(labels::delete_label)
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
        .service(revisions::rollback_instance_revision);
}

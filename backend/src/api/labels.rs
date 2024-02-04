use std::sync::Arc;

use crate::model::{LabelType, YakManRole};
use crate::services::StorageService;
use crate::{
    api::is_alphanumeric_kebab_case, error::CreateLabelError, error::YakManApiError,
    middleware::roles::YakManRoleBinding,
};
use actix_web::{get, put, web, Responder};
use actix_web_grants::permissions::AuthDetails;

/// Get all labels
#[utoipa::path(responses((status = 200, body = Vec<LabelType>)))]
#[get("/v1/labels")]
pub async fn get_labels(
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let data = storage_service.get_labels().await?;
    return Ok(web::Json(data));
}

/// Create a new label
#[utoipa::path(request_body = LabelType, responses((status = 200, body = String)))]
#[put("/v1/labels")]
pub async fn create_label(
    auth_details: AuthDetails<YakManRoleBinding>,
    label_type: web::Json<LabelType>,
    storage_service: web::Data<Arc<dyn StorageService>>,
) -> Result<impl Responder, YakManApiError> {
    let mut label_type = label_type.into_inner();
    label_type.name = label_type.name.to_lowercase();

    if !YakManRoleBinding::has_any_global_role(
        vec![YakManRole::Admin, YakManRole::Approver],
        &auth_details.permissions,
    ) {
        return Err(YakManApiError::forbidden());
    }

    if !is_alphanumeric_kebab_case(&label_type.name) {
        return Err(YakManApiError::bad_request(
            "Invalid label name. Must be alphanumeric kebab case",
        ));
    }

    return match storage_service.create_label(label_type).await {
        Ok(()) => Ok(web::Json(())),
        Err(e) => match e {
            CreateLabelError::DuplicateLabelError { name: _ } => {
                Err(YakManApiError::bad_request("Duplicate label"))
            }
            CreateLabelError::EmptyOptionsError => Err(YakManApiError::bad_request(
                "Label must have at least 1 option",
            )),
            CreateLabelError::StorageError { message } => {
                log::error!("Failed to create label, error: {message}");
                Err(YakManApiError::server_error("Failed to create label"))
            }
        },
    };
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::*;
    use actix_web::{test, web::Data, App};
    use actix_web_grants::GrantsMiddleware;
    use anyhow::Result;
    use serde_json::Value;

    fn foo_label() -> LabelType {
        LabelType {
            name: "foo".to_string(),
            description: "my foo label".to_string(),
            options: vec!["foo-1".to_string(), "foo-2".to_string()],
        }
    }

    fn bar_label() -> LabelType {
        LabelType {
            name: "bar".to_string(),
            description: "my bar label".to_string(),
            options: vec!["bar-1".to_string(), "bar-2".to_string()],
        }
    }

    #[actix_web::test]
    async fn get_labels_should_return_labels() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        state.service.create_label(foo_label()).await?;
        state.service.create_label(bar_label()).await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(get_labels),
        )
        .await;
        let req = test::TestRequest::get().uri("/v1/labels").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let value: Value = body_to_json_value(resp).await?;

        let first = &value.as_array().unwrap()[0];
        assert_eq!("foo", first["name"]);

        let second = &value.as_array().unwrap()[1];
        assert_eq!("bar", second["name"]);

        Ok(())
    }

    #[actix_web::test]
    async fn create_label_should_create_labels_properly() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        // Make sure we are starting with no labels
        assert_eq!(0, state.service.get_labels().await?.len());

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_label),
        )
        .await;
        let req = test::TestRequest::put()
            .uri("/v1/labels")
            .set_json(foo_label())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        let labels = state.service.get_labels().await?;

        // Make sure only 1 label was created
        assert_eq!(1, labels.len());

        let label = &labels[0];
        assert_eq!("foo", label.name);
        assert_eq!("my foo label", label.description);
        assert_eq!(vec!["foo-1", "foo-2"], label.options);

        Ok(())
    }

    #[actix_web::test]
    async fn create_label_should_not_allow_invalid_label_names() -> Result<()> {
        prepare_for_actix_test()?;

        let state = test_state_manager().await?;

        let app = test::init_service(
            App::new()
                .app_data(Data::new(state.clone()))
                .wrap(GrantsMiddleware::with_extractor(fake_roles::admin_role))
                .service(create_label),
        )
        .await;

        let mut bad_foo_label = foo_label();
        bad_foo_label.name = "foo but not valid".to_string();

        let req = test::TestRequest::put()
            .uri("/v1/labels")
            .set_json(bad_foo_label)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());

        // Make sure no labels were created
        assert_eq!(0, state.service.get_labels().await?.len());

        Ok(())
    }
}

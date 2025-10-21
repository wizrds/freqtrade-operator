use axum::{
    extract::Extension,
    response::IntoResponse,
    routing::post,
    Router,
    Json,
};
use std::sync::Arc;
use kube::core::{admission::{AdmissionRequest, AdmissionResponse, AdmissionReview}, DynamicObject};

use ft_operator_common::state::State;

use crate::admission::bot::validate_bot_crd;

pub fn router() -> Router {
    Router::new().route("/freqtrade.io/bot/validate", post(validate_bot_crd_endpoint))
}

async fn validate_bot_crd_endpoint(Extension(_state): Extension<Arc<State>>, payload: Json<AdmissionReview<DynamicObject>>) -> impl IntoResponse {
    let request: AdmissionRequest<DynamicObject> = match payload.0.try_into() {
        Ok(request) => request,
        Err(err) => {
            return Json(AdmissionResponse::invalid(err.to_string()).into_review());
        }
    };
    // Defaults to allow
    let mut response = AdmissionResponse::from(&request);
    
    // Validate any reserved config keys, and deny if found
    match validate_bot_crd(&request.object.unwrap()) {
        Ok(_) => (),
        Err(err) => {
            response = response.deny(err.to_string());
        }
    }

    // Convert the response to a review and return it
    Json(response.into_review())
}
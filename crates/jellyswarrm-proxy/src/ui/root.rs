use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::{Level, Messages};
use tracing::{error, info};

use crate::{
    ui::{
        auth::{AuthenticatedUser, UserRole},
        JellyfinUiVersion, JELLYFIN_UI_VERSION,
    },
    AppState,
};

#[derive(Template)]
#[template(path = "user/index.html")]
pub struct UserIndexTemplate {
    pub version: Option<String>,
    pub ui_route: String,
    pub root: Option<String>,
    pub jellyfin_ui_version: Option<JellyfinUiVersion>,
    /// Pending flash messages as (text, background colour, icon class).
    pub flash: Vec<(String, String, String)>,
}

#[derive(Template)]
#[template(path = "admin/index.html")]
pub struct AdminIndexTemplate {
    pub version: Option<String>,
    pub ui_route: String,
    pub root: Option<String>,
    pub jellyfin_ui_version: Option<JellyfinUiVersion>,
    /// Pending flash messages as (text, background colour, icon class).
    pub flash: Vec<(String, String, String)>,
}

/// Root/home page
pub async fn index(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    messages: Messages,
) -> impl IntoResponse {
    let flash: Vec<(String, String, String)> = messages
        .into_iter()
        .map(|m| {
            let (colour, icon) = if matches!(m.level, Level::Error | Level::Warning) {
                ("#c62828", "fa-exclamation-circle")
            } else {
                ("#2e7d32", "fa-check-circle")
            };
            (m.to_string(), colour.to_string(), icon.to_string())
        })
        .collect();
    let response = if user.role == UserRole::User {
        let template = UserIndexTemplate {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ui_route: state.get_ui_route().await,
            root: state.get_url_prefix().await,
            jellyfin_ui_version: JELLYFIN_UI_VERSION.clone(),
            flash: flash.clone(),
        };

        match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                error!("Failed to render index template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
            }
        }
    } else {
        info!("Rendering admin dashboard for {}", user.username);
        let template = AdminIndexTemplate {
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            ui_route: state.get_ui_route().await,
            root: state.get_url_prefix().await,
            jellyfin_ui_version: JELLYFIN_UI_VERSION.clone(),
            flash: flash.clone(),
        };

        match template.render() {
            Ok(html) => Html(html).into_response(),
            Err(e) => {
                error!("Failed to render index template: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response()
            }
        }
    };
    response
}

use utoipa::{OpenApi, Modify, openapi::security::{SecurityScheme, ApiKey, ApiKeyValue}};
// use utoipa::openapi::security::Http

#[derive(OpenApi)]
#[openapi(
    info(title = "Little CDN"),
    paths(
        // Auth
        crate::controllers::user_controller::login::login,
        crate::controllers::user_controller::sign_up::sign_up,

        // Files
        crate::controllers::files_controller::add_file::add_file,
        crate::controllers::files_controller::get_file::get_file,
        crate::controllers::files_controller::list_files::list_files,
        crate::controllers::files_controller::remove_file::remove_file,
    ),
    components(
        schemas(
            crate::controllers::user_controller::LoginData,
            crate::controllers::user_controller::LoginAuth,
        ),
        schemas(
            crate::controllers::files_controller::FileQuery,
            crate::controllers::files_controller::FileEntity,
            lib::error::AppError,
            lib::error::WebSocketError,
            lib::error::ApiError,
        ),
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Little CDN", description = "Little CDN documentation")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
        components.add_security_scheme(
            "Authorization",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
        )
    }
}


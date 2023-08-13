use actix_web::web::{self, ServiceConfig};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub mod login;
pub mod sign_up;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/login").route(web::post().to(login::login)))
            .service(web::resource("/signup").route(web::post().to(sign_up::sign_up)));
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct LoginData {
    #[schema(example = "root")]
    pub username: String,
    #[schema(example = "P@ssW0r3")]
    pub password: String,
}

#[derive(Serialize, ToSchema)]
pub struct LoginAuth {
    #[schema(example = "9VAZNG7tHdJkt1oAECRVNYfrG5AJEpMyTaT8lFqhDeRvDGVUGQqiGqBt73pY")]
    pub token: String,
}

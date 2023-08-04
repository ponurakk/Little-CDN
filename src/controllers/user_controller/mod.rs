use actix_web::web::{self, ServiceConfig};
use serde::Deserialize;

pub mod login;
pub mod sign_up;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/login").route(web::post().to(login::login)))
            .service(web::resource("/signup").route(web::post().to(sign_up::sign_up)));
    }
}

#[derive(Deserialize)]
pub struct LoginData {
    pub username: String,
    pub password: String,
}

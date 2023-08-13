pub mod add_file;
pub mod list_files;
pub mod get_file;
pub mod remove_file;

use actix_web::web::{self, ServiceConfig};
use actix_web_lab::middleware::from_fn;
use lib::auth_middleware;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/files")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(add_file::add_file))
                .route(web::get().to(list_files::list_files)))
            .service(web::resource("/file")
                .wrap(from_fn(auth_middleware))
                .route(web::get().to(get_file::get_file))
                .route(web::delete().to(remove_file::remove_file)));
    }
}

#[derive(Deserialize, ToSchema)]
pub struct FileQuery {
    filename: String,
}

#[derive(Serialize, ToSchema)]
pub struct FileEntity {
    pub filename: String,
    pub filetype: String,
    pub size: i64,
    pub created_at: String,
    pub updated_at: String,
}

pub mod add_file;
pub mod list_files;
pub mod get_file;

use actix_web::web::{self, ServiceConfig};
use actix_web_lab::middleware::from_fn;
use lib::auth_middleware;
use serde::Deserialize;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/files")
                .wrap(from_fn(auth_middleware))
                .route(web::post().to(add_file::add_file))
                .route(web::get().to(list_files::list_files)))
            .service(web::resource("/file")
                .wrap(from_fn(auth_middleware))
                .route(web::get().to(get_file::get_file)));
    }
}

#[derive(Deserialize)]
pub struct FileQuery {
    filename: String,
}

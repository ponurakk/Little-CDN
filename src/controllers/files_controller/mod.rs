pub mod add_file;

use actix_web::web::{ServiceConfig, self};

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/files").route(web::post().to(add_file::add_file)));
    }
}

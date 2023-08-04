pub mod add_file;

use actix_web::web::{self, ServiceConfig};
use actix_web_lab::middleware::from_fn;
use lib::auth_middleware;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/files").wrap(from_fn(auth_middleware)).route(web::post().to(add_file::add_file)));
    }
}

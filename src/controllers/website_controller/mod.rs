use actix_web::{web::{ServiceConfig, self}, http::header::ContentType, HttpResponse, Responder};
use lib::error::ApiError;

use crate::AppState;
use self::macros::html_controller;

pub fn configure() -> impl FnOnce(&mut ServiceConfig) {
    html_controller!(index, "index.html");

    |config: &mut ServiceConfig| {
        config
            .service(web::resource("/").route(web::get().to(index)));
    }
}

mod macros {
    macro_rules! html_controller {
        ( $function_name:ident, $template_name:expr $(, $context:tt)* ) => {
            async fn $function_name(data: web::Data<AppState>) -> Result<impl Responder, ApiError> {
                let tmpl = &data.tera;
                let context = tera::Context::new();
                $(context.insert$context;)*
                let s = tmpl.render($template_name, &context)?;

                Ok(HttpResponse::Ok().content_type(ContentType::html()).body(s))
            }
        }
    }
    pub(super) use html_controller;
}

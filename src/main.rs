#[macro_use]
extern crate log;

mod cmd;
mod docs;
pub mod util;

mod controllers;

use actix_cors::Cors;
use actix_web::{HttpServer, App, web, HttpResponse, http::{header::{self, ContentType}, StatusCode}, middleware::{self, ErrorHandlers, ErrorHandlerResponse}, dev::ServiceResponse, HttpRequest};
use actix_web_actors::ws;
use actix_files as afs;
use lib::{Config, AppState, create_root_user, load_html, websocket_server::WebSocket, error::AppError, DEFAULT_TARGET};
use migration::{Migrator, MigratorTrait};
use sea_orm::Database;
use tera::Tera;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::docs::ApiDoc;

#[actix_web::main]
async fn main() -> Result<(), AppError> {
    let config: Config = cmd::init()?;

    let openapi = ApiDoc::openapi();
    let conn = Database::connect("sqlite:./sqlite.db?mode=rwc").await?;
    let tera = Tera::new("debug_templates/views/**/*")?;

    Migrator::up(&conn, None).await?;

    create_root_user(&conn).await?;
    load_html()?;

    info!("Starting HTTP server at {}:{}", &config.address, &config.port);
    let bind = (config.address, config.port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        let state: AppState = AppState::new(conn.clone(), tera.clone(), config.clone());

        let mut app = App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default().log_target(DEFAULT_TARGET))
            .app_data(web::Data::new(state))
            .default_service(web::to(HttpResponse::NotFound))
            .route("/docs", web::get().to(|| async { HttpResponse::Found().insert_header((header::LOCATION, "/docs/")).finish() }))
            .service(
                web::scope("/api")
                    .configure(controllers::files_controller::configure())
                    .configure(controllers::user_controller::configure())
            )
            .service(web::resource("/ws").route(web::get().to(websocket_handler)))
            .service(SwaggerUi::new("/docs/{_:.*}").url("/api-doc/openapi.json", openapi.clone()));

        if !&config.stop_web {
            app = app.service(
                web::scope("")
                    .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found))
                    .configure(controllers::website_controller::configure())
                    .service(afs::Files::new("/assets", "templates/assets")),
            )
        }

        app
    })
    .bind(bind)?
    .run()
    .await?;

    Ok(())
}

#[allow(clippy::unused_async)]
async fn websocket_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    ws::start(WebSocket::new(req.clone()), &req, stream)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> actix_web::Result<ErrorHandlerResponse<B>> where HttpResponse<B>: std::convert::From<HttpResponse> {
    let response = get_error_response::<B>(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(res.into_parts().0, response.map_into_left_body())))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse<B> where HttpResponse<B>: std::convert::From<HttpResponse> {
    let request = res.request();
    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(e.to_string()).into()
    };

    let tera = request.app_data::<web::Data<AppState>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let tera = tera.tera.clone();
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("index.html", &context);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body).into(),
                Err(e) => {
                    error!("{:?}", e);
                    fallback(error)
                }
            }
        }
        None => fallback(error),
    }
}

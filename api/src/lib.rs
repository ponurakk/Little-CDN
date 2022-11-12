use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    http::{ header::ContentType, StatusCode },
    middleware::{ self, ErrorHandlerResponse, ErrorHandlers },
    web, App, HttpResponse, HttpServer, Result,
};
use actix_files as afs;
use tera::Tera;
use rust_embed::RustEmbed;

mod controllers;
use controllers::{ user_controller, admin_controller };

#[derive(Debug, Clone)]
pub struct AppState {
    conn: PathBuf,
    tera: Tera,
    files_path: PathBuf,
    config: Config,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub stop_web: bool,
    pub address: String,
    pub port: u16,
    pub log: u8,
    pub dir: PathBuf,
    pub db: PathBuf,
    pub disable_login: bool,
    pub clear_database: bool,
}

#[actix_web::main]
pub async fn main(config: Config) -> std::io::Result<()> {

    if config.log > 0 {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    let dir = format!("./{}", config.dir.display());
    std::fs::create_dir_all(&dir)?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // todo!("Cusom file name");

    let load = load_html(&config);
    match load {
        Ok(v) => v,
        Err(e) => {
            panic!("Loading Website resulted in error {:#?}", e);
        }
    }

    let bind = (config.address.clone(), config.port.clone());
    println!("Listening on: {}:{}", bind.0, bind.1);

    HttpServer::new(move || {
        let tera = Tera::new("templates/**/*").unwrap();
        let logger = middleware::Logger::new("\"%a %s %r\" %b bytes %T s");
        let state = AppState {
            conn: config.db.clone(),
            tera,
            files_path: config.dir.clone(),
            config: config.clone(),
        };

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(logger)
            .service(
                web::scope("/")
                    .service(user_controller::index)
                    .service(user_controller::save_file)
            )
            .service(
                web::scope("")
                    .service(afs::Files::new("/files", &dir).show_files_listing())
                    .service(afs::Files::new("/style", "./stylesheets"))
            )
            .service(
                web::scope("")
                    .wrap(error_handlers())
            )
    })
        .bind(bind)?
        .run()
        .await
}

fn load_html(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    #[derive(RustEmbed)]
    #[folder = "stylesheets/"]
    #[prefix = "stylesheets/"]
    struct Css;

    #[derive(RustEmbed)]
    #[folder = "templates/"]
    #[prefix = "templates/"]
    struct HTML;

    for file in Css::iter() {
        let index = Css::get(file.as_ref()).expect("Error in reading css file");
        let content = std::str::from_utf8(index.data.as_ref())?.as_bytes();
        let mut new_file = File::create(file.as_ref())?;
        new_file.write_all(content)?;
        if config.log > 1 {
            println!("Created: {}", file.as_ref());
        }
    }

    for file in HTML::iter() {
        let index = HTML::get(file.as_ref()).expect("Error in reading css file");
        let content = std::str::from_utf8(index.data.as_ref())?.as_bytes();
        let mut new_file = File::create(file.as_ref())?;
        new_file.write_all(content)?;
        if config.log > 1 {
            println!("Created: {}", file.as_ref());
        }
    }
    Ok(())
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<BoxBody> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<BoxBody>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(ServiceResponse::new(
        res.into_parts().0,
        response.map_into_left_body(),
    )))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> HttpResponse {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        HttpResponse::build(res.status())
            .content_type(ContentType::plaintext())
            .body(e.to_string())
    };

    let tera = request.app_data::<web::Data<Tera>>().map(|t| t.get_ref());
    match tera {
        Some(tera) => {
            let mut context = tera::Context::new();
            context.insert("error", error);
            context.insert("status_code", res.status().as_str());
            let body = tera.render("error.html", &context);

            match body {
                Ok(body) => HttpResponse::build(res.status())
                    .content_type(ContentType::html())
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
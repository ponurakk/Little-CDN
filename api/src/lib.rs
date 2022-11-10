use std::io::Write;
use std::collections::HashMap;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error,
    get, post,
    http::{ header::ContentType, StatusCode },
    middleware::{ self, ErrorHandlerResponse, ErrorHandlers },
    web, App, Error, HttpResponse, HttpServer, Responder, Result,
};
use actix_files as fs;
use actix_web_lab::respond::Html;
use actix_multipart::Multipart;
use tera::Tera;

use chrono;
use futures_util::TryStreamExt;

#[post("")]
async fn save_file(mut payload: Multipart) -> Result<HttpResponse, Error> {
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition();

        if content_disposition.get_name() == Some("file") {
            let dt = chrono::offset::Utc::now().format("Uploaded_%Y-%m-%d_%H-%M-%S").to_string();

            let filename = content_disposition
                .get_filename()
                .unwrap()
                .split(".");
            let vec = filename.collect::<Vec<&str>>();
            let filepath = format!("./tmp/{}.{}", dt, vec[vec.len()-1]);

            // File::create is blocking operation, use threadpool
            let mut f = web::block(|| std::fs::File::create(filepath)).await??;

            // Field in turn is stream of *Bytes* object
            while let Some(chunk) = field.try_next().await? {
                // filesystem operations are blocking, we have to use threadpool
                f = web::block(move || f.write_all(&chunk).map(|_| f)).await??;
            }
        } else if content_disposition.get_name() == Some("token") {
            todo!("Auth")
        }
    }

    Ok(HttpResponse::Ok().into())
}

// store tera template in application state
#[get("")]
async fn index(
    tmpl: web::Data<tera::Tera>,
    query: web::Query<HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let s = if let Some(name) = query.get("name") {
        // submitted form
        let mut ctx = tera::Context::new();
        ctx.insert("name", name);
        ctx.insert("text", "Welcome!");
        tmpl.render("post.html", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    } else {
        tmpl.render("index.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?
    };

    Ok(Html(s))
}

#[actix_web::main]
pub async fn main(
    address: String,
    port: u16,
    log: bool,
    dir_name: String,
) -> std::io::Result<()> {

    if log {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    let dir = format!("./{}", dir_name);
    std::fs::create_dir_all(&dir)?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // todo!("Cusom file name");

    println!("Listening on: {}:{}", address, port);

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        let logger = middleware::Logger::new("\"%a %s %r\" %b bytes %T s");

        App::new()
            .app_data(web::Data::new(tera))
            .wrap(logger)
            .service(
                web::scope("/")
                    .service(index)
                    .service(save_file)
            )
            .service(
                web::scope("")
                    .service(fs::Files::new("/files", &dir).show_files_listing())
                    // .service(fs::Files::new("/static", "./static").show_files_listing())
            )
            .service(
                web::scope("")
                    .wrap(error_handlers())
            )
    })
        .bind((address, port))?
        .run()
        .await
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
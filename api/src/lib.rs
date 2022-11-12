use std::collections::HashMap;
use std::path::PathBuf;
use std::fs::rename;

use actix_web::{
    body::BoxBody,
    dev::ServiceResponse,
    error,
    get, post,
    http::{ header::ContentType, StatusCode },
    middleware::{ self, ErrorHandlerResponse, ErrorHandlers },
    web, App, Error, HttpResponse, HttpServer, Responder, Result,
};
use actix_files as afs;
use actix_web_lab::respond::Html;
use tera::Tera;

use chrono;

use actix_easy_multipart::MultipartForm;
use actix_easy_multipart::text::Text;
use actix_easy_multipart::tempfile::Tempfile;


#[derive(Debug, Clone)]
struct AppState {
    conn: PathBuf,
    tera: Tera,
    files_path: PathBuf,
}

#[derive(MultipartForm)]
struct Upload {
    token: Text<String>,
    file: Tempfile,
}

#[post("")]
async fn save_file(
    form: MultipartForm<Upload>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let dt = chrono::offset::Utc::now().format("Uploaded_%Y-%m-%d_%H-%M-%S").to_string();
    let filename = String::from(&form.file.file_name.clone().unwrap());
    let vec = filename.split(".").collect::<Vec<&str>>();
    let filepath = format!("./{}/{}.{}", &data.files_path.display(), dt, vec[vec.len()-1]);
    rename(&form.file.file.path(), filepath)?;

    // let db = sled::open(&data.conn)?;
    // let user = "";
    // db.insert(key, );

    Ok(HttpResponse::Created().finish())
}

// store tera template in application state
#[get("")]
async fn index(
    data: web::Data<AppState>,
    query: web::Query<HashMap<String, String>>,
) -> Result<impl Responder, Error> {
    let tmpl = &data.tera;
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
    stop_web: bool,
    address: String,
    port: u16,
    log: bool,
    dir_name: PathBuf,
    db_path: PathBuf,
    clear_db: bool,
    disable_login: bool,
) -> std::io::Result<()> {
    let db = sled::open(&db_path)?;
    let key = "authorize";

    let val = db.get(key)?;
    println!("{:#?}", std::str::from_utf8(&val.unwrap()).unwrap());

    if log {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }

    let dir = format!("./{}", dir_name.display());
    std::fs::create_dir_all(&dir)?;
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // todo!("Cusom file name");

    println!("Listening on: {}:{}", address, port);

    HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
        let logger = middleware::Logger::new("\"%a %s %r\" %b bytes %T s");
        let state = AppState {
            conn: db_path.clone(),
            tera,
            files_path: dir_name.clone(),
        };

        App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(logger)
            .service(
                web::scope("/")
                    .service(index)
                    .service(save_file)
            )
            .service(
                web::scope("")
                    .service(afs::Files::new("/files", &dir).show_files_listing())
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
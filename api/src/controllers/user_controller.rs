use actix_web::{ web, get, post, HttpResponse, Error, error };
use actix_easy_multipart::MultipartForm;
use actix_easy_multipart::text::Text;
use actix_easy_multipart::tempfile::Tempfile;

use std::fs::rename;
use chrono;

use crate::AppState;

#[derive(MultipartForm)]
pub struct Upload {
    token: Text<String>,
    user: Text<String>,
    file: Tempfile,
}

#[get("")]
pub async fn index(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    if data.config.stop_web {
        return Ok(HttpResponse::NotFound().finish())
    }
    let tmpl = &data.tera;
    let s = tmpl.render("index.html", &tera::Context::new())
        .map_err(|e| {
            println!("{:#?}", e);
            error::ErrorInternalServerError("Template error")
        })?;

    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[post("")]
pub async fn save_file(
    form: MultipartForm<Upload>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let dt = chrono::offset::Utc::now().format("Uploaded_%Y-%m-%d_%H-%M-%S").to_string();
    let filename = String::from(&form.file.file_name.clone().unwrap());
    let vec = filename.split(".").collect::<Vec<&str>>();
    let filepath = format!("./{}/{}.{}", &data.files_path.display(), dt, vec[vec.len()-1]);
    rename(&form.file.file.path(), filepath)?;

    println!("{:#?}", &form.user);
    println!("{:#?}", &form.token);

    // let db = sled::open(&data.conn)?;
    // db.insert(&form.name, &form.token.unwrap());

    // println!("{:#?}", db.iter().into());

    Ok(HttpResponse::Created().finish())
}

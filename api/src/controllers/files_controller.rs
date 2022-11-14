use std::fs;
use actix_web::{web, get, post, delete, HttpResponse, Error, error };
use actix_web::http::header::ContentType;
use actix_easy_multipart::MultipartForm;
use actix_easy_multipart::text::Text;
use actix_easy_multipart::tempfile::Tempfile;

use serde::Deserialize;

use std::fs::rename;
use chrono;
use serde_json::json;

use crate::AppState;
use crate::models::files::Files;
use crate::models::files::File;
use crate::utils::auth::authorize;

#[derive(MultipartForm)]
pub struct Upload {
    pub token: Text<String>,
    pub user: Text<String>,
    pub file: Tempfile,
}

#[get("/")]
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

#[post("/files")]
pub async fn save_file(
    form: MultipartForm<Upload>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    match authorize(&db, form.0.user.clone(), form.0.token.clone()) {
        Ok(v) => v,
        Err(_) => {
            return Ok(
                HttpResponse::Unauthorized()
                    .content_type(ContentType::json())
                    .json(json!({ "message": "User or token is invalid" }))
            );
        }
    };
    let files_tree = db.open_tree(b"files").unwrap();

    let mut user_files = match files_tree.get(&form.0.user.clone()).unwrap() {
        Some(v) => {
            let decoded_file: Files = bincode::deserialize(&v[..]).unwrap();
            Files {
                user: form.0.user.clone(),
                files: decoded_file.files,
                files_count: decoded_file.files_count,
            }
        },
        None => {
             Files {
                user: form.0.user.clone(),
                files: vec![],
                files_count: 0,
            }
        }
    };

    let dt = chrono::offset::Utc::now().format("Uploaded_%Y-%m-%d_%H-%M-%S").to_string();
    let filename = String::from(&form.file.file_name.clone().unwrap());
    let vec = filename.split('.').collect::<Vec<&str>>();
    let filepath = format!("./{}/{}/{}.{}", &data.files_path.display(), &form.0.user.clone(), dt, vec[vec.len()-1]);
    rename(form.file.file.path(), filepath)?;

    user_files.files_count += 1;
    user_files.files.push(
        File {
            original: filename.clone(),
            changed: format!("{}.{}", dt, vec[vec.len()-1])
        }
    );
    let encoded = bincode::serialize(&user_files).unwrap();
    files_tree.insert(form.0.user.clone(), encoded).unwrap();

    Ok(HttpResponse::Created().finish())
}

#[get("/files/{user}")]
pub async fn get_files(
    path: web::Path<String>,
    body: String,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let auth = match authorize(&db, path.into_inner(), body) {
        Ok(v) => v,
        Err(_) => {
            return Ok(
                HttpResponse::Unauthorized()
                    .content_type(ContentType::json())
                    .json(json!({ "message": "User or token is invalid" }))
            );
        }
    };

    let files_tree = db.open_tree(b"files").unwrap();
    let files = files_tree.get(auth.name).unwrap().unwrap();
    let files: Files = bincode::deserialize(&files[..]).unwrap();

    Ok(
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .json(json!{ files })
    )
}

#[derive(Deserialize)]
pub struct DeleteFile {
    token: String,
    file: String,
}

#[delete("/files/{user}")]
pub async fn delete_file(
    path: web::Path<String>,
    body: web::Json<DeleteFile>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let auth = match authorize(&db, path.clone(), body.token.clone()) {
        Ok(v) => v,
        Err(_) => {
            return Ok(
                HttpResponse::Unauthorized()
                    .content_type(ContentType::json())
                    .json(json!({ "message": "User or token is invalid" }))
            );
        }
    };

    let files_tree = db.open_tree(b"files").unwrap();
    let files = files_tree.get(auth.name).unwrap().unwrap();
    let mut files: Files = bincode::deserialize(&files[..]).unwrap();

    let mut removed = false;
    files.files.retain(|f| {
            if f.changed == body.file {
                removed = true;
            }
            f.changed != body.file
        }
    );

    if removed {
        files.files_count -= 1;
        let encoded = bincode::serialize(&files).unwrap();
        files_tree.insert(&path.into_inner(), &*encoded).unwrap();
        fs::remove_file(format!("./{}/{}/{}", &data.files_path.display(), files.user, &body.file)).unwrap();
        return Ok(
            HttpResponse::Accepted().finish()
        );
    }

    Ok(
        HttpResponse::BadRequest()
            .content_type(ContentType::json())
            .json(json!({ "message": "File with that name doesn't exists" }))
    )
}

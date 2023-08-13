use std::{fs::{self, File}, io::Read};

use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use super::FileQuery;

/// Download specified file
/// # cURL example:
///
/// ---
/// ```bash
/// curl -X GET 'http://127.0.0.1:3000/api/file?filename=File1.zip' \
/// -H 'Authorization: Bearer 9VAZNG7tHdJkt1oAECRVNYfrG5AJEpMyTaT8lFqhDeRvDGVUGQqiGqBt73pY' \
/// -o File1.zip
/// ```
#[utoipa::path(
    get,
    path = "/api/file",
    tag = "Files",
    security(
        ("Authorization" = [])
    ),
    params(
        ("filename" = String, Query, description = "Name of the file"),
    ),
    responses(
        (status = 200, body = Vec<u8>, description = "Requested binary", content_type = "application/octet-stream"),
        (status = 404, description = "Some value doesn't exist"),
    )
)]
pub async fn get_file(
    query: web::Query<FileQuery>,
    data: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    let user_guard = data.user.lock().await;
    let user = user_guard.as_ref().ok_or(AppError::NoneValue("User"))?;

    let file = files::Entity::find()
        .filter(files::Column::UserId.eq(user.id))
        .filter(files::Column::Filename.eq(&query.filename))
        .one(&data.conn)
        .await?;

    match file {
        Some(v) => {
            let path = format!("{}/{}/{}", data.config.dir.display(), user.uuid, v.filename);
            let mut f = File::open(&path)?;
            let metadata = fs::metadata(&path)?;
            let mut buffer = vec![0; metadata.len() as usize];
            f.read(&mut buffer)?;

            Ok(HttpResponse::Ok().body(buffer))
        }
        None => Err(AppError::ApiError(lib::error::ApiError::FileNotFound))
    }
}

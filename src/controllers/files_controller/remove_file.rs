use std::fs;

use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use crate::util::User;

use super::FileQuery;

/// Remove specified file
/// # cURL example:
///
/// ---
/// ```bash
/// curl -X DELETE 'http://127.0.0.1:3000/api/file?filename=File1.zip' \
/// -H 'Authorization: Bearer 9VAZNG7tHdJkt1oAECRVNYfrG5AJEpMyTaT8lFqhDeRvDGVUGQqiGqBt73pY'
/// ```
#[utoipa::path(
    delete,
    path = "/api/file",
    tag = "Files",
    security(
        ("Authorization" = [])
    ),
    params(
        ("filename" = String, Query, description = "Name of the file"),
    ),
    responses(
        (status = 200, description = "Seccesfully removed file"),
        (status = 404, description = "Some value doesn't exist"),
    )
)]

pub async fn remove_file(
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
            user.remove_file(&data.conn, v.clone()).await?;
            let path = format!("{}/{}/{}", data.config.dir.display(), user.uuid, v.filename);
            fs::remove_file(path)?;
            Ok(HttpResponse::Ok().finish())
        }
        None => Err(AppError::ApiError(lib::error::ApiError::FileNotFound))
    }
}

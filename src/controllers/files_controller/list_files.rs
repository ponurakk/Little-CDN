use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use super::FileEntity;

/// List every file user has
/// # cURL example:
///
/// ---
/// ```bash
/// curl -X GET http://127.0.0.1:3000/api/files \
/// -H 'Authorization: Bearer 9VAZNG7tHdJkt1oAECRVNYfrG5AJEpMyTaT8lFqhDeRvDGVUGQqiGqBt73pY'
/// ```
#[utoipa::path(
    get,
    path = "/api/files",
    tag = "Files",
    security(
        ("Authorization" = [])
    ),
    responses(
        (status = 200, body = Vec<FileEntity>, description = "List of files"),
        (status = 404, description = "Some value doesn't exist"),
    )
)]
pub async fn list_files(
    data: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    let user_guard = data.user.lock().await;
    let user = user_guard.as_ref().ok_or(AppError::NoneValue("User"))?;

    let files = files::Entity::find()
        .filter(files::Column::UserId.eq(user.id))
        .all(&data.conn)
        .await?;

    let files_entities: Vec<FileEntity> = files.iter().map(|file| {
        FileEntity {
            filename: file.filename.clone(),
            filetype: file.filetype.clone(),
            size: file.size,
            created_at: file.created_at.clone(),
            updated_at: file.updated_at.clone(),
        }
    }).collect();

    Ok(HttpResponse::Ok().json(files_entities))
}

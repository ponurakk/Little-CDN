use std::fs;

use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::{files, auth, users}};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ModelTrait, Set};

/// Download specified file
/// # cURL example:
///
/// ---
/// ```bash
/// curl -X DELETE http://127.0.0.1:3000/api/remove_user \
/// -H 'Authorization: Bearer 9VAZNG7tHdJkt1oAECRVNYfrG5AJEpMyTaT8lFqhDeRvDGVUGQqiGqBt73pY'
/// ```
#[utoipa::path(
    get,
    path = "/api/remove_user",
    tag = "Auth",
    security(
        ("Authorization" = [])
    ),
    responses(
        (status = 200, description = "Succesfully removed user account"),
        (status = 404, description = "Some value doesn't exist"),
    )
)]
pub async fn remove_user(
    data: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    let user_guard = data.user.lock().await;
    let user = user_guard.as_ref().ok_or(AppError::NoneValue("User"))?;
    fs::remove_dir_all(format!("{}/{}", data.config.dir.display(), user.uuid))?;

    files::Entity::delete_many()
        .filter(files::Column::UserId.eq(user.id))
        .exec(&data.conn)
        .await?;

    let auth = user.find_related(auth::Entity)
        .one(&data.conn)
        .await?
        .ok_or(AppError::NoneValue("auth"))?;

    auth.delete(&data.conn).await?;

    users::Entity::delete_by_id(user.id).exec(&data.conn).await?;

    Ok(HttpResponse::Ok().finish())
}

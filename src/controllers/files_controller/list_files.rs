use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use serde_json::json;

pub async fn list_files(
    data: web::Data<AppState>
) -> Result<impl Responder, AppError> {
    let user_guard = data.user.lock().await;
    let user = user_guard.as_ref().ok_or(AppError::NoneValue("User"))?;

    let files = files::Entity::find()
        .filter(files::Column::UserId.eq(user.id))
        .all(&data.conn)
        .await?;

    Ok(HttpResponse::Ok().json(json!(files)))
}

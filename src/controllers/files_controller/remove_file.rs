use std::{fs::{self, File}, io::Read};

use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use crate::util::User;

use super::FileQuery;

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

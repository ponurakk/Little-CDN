use std::{fs::{self, File}, io::Read};

use actix_web::{web, Responder, HttpResponse};
use lib::{AppState, error::AppError, entities::files};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};

use super::FileQuery;

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

use std::{
    fs::{self, OpenOptions},
    io::Write,
};

use crate::util::User;
use actix_multipart::Multipart;
use actix_web::{error, web, HttpRequest, HttpResponse, Responder};
use futures_util::stream::StreamExt;
use lib::{error::AppError, AppState, DEFAULT_TARGET};

pub async fn add_file(
    req: HttpRequest,
    mut payload: Multipart,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let connection = req.connection_info().clone();
    let host = connection.peer_addr().unwrap_or("unknown host");
    let user_guard = data.user.lock().await;
    let user = user_guard.as_ref().ok_or(AppError::NoneValue("User"))?;

    let mut files = Vec::<Vec<u8>>::new();
    let mut filenames = Vec::<String>::new();
    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content = field.content_disposition().clone();
        let filename = content
            .get_filename()
            .ok_or(AppError::NoneValue("filename"))?
            .to_owned();
        filenames.push(filename);
        let mut bytes = Vec::<u8>::new();
        while let Some(chunk) = field.next().await {
            bytes.append(&mut chunk?.to_vec());
        }
        if bytes.is_empty() {
            log::warn!(target: DEFAULT_TARGET, "{} sent zero bytes", host);
            return Err(AppError::ActixError(error::ErrorBadRequest(
                "Invalid file size",
            )));
        }
        files.push(bytes);
    }

    let mut total_size: i64 = 0;

    for (i, file) in files.iter().enumerate() {
        let file_size = file.len() as i64;
        if user.has_free_space(file_size) {
            let filename = filenames.get(i).ok_or(AppError::NoneValue("filename"))?;
            total_size += file_size;
            fs::create_dir_all(format!("{}/{}", data.config.dir.display(), user.uuid))?;
            let path = format!("{}/{}/{}", data.config.dir.display(), user.uuid, filename);
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(mut v) => v.write_all(file)?,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::AlreadyExists {
                        return Err(AppError::ApiError(lib::error::ApiError::AlreadyExists));
                    }
                    return Err(AppError::IoError(e));
                }
            }
            user.add_file(&data.conn, file, filename, file_size).await?;
        } else {
            return Err(AppError::ApiError(lib::error::ApiError::LowStorage));
        }
    }

    user.set_storage(&data.conn, total_size).await?;

    Ok(HttpResponse::Ok().finish())
}

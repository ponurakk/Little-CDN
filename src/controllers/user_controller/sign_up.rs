use actix_web::{web, HttpResponse, Responder};
use chrono::Duration;
use lib::{
    entities::{auth, users},
    error::AppError,
    make_token, AppState,
};
use pwhash::bcrypt;
use sea_orm::{ActiveModelTrait, Set};
use uuid::Uuid;

use super::{LoginData, LoginAuth};

/// SignUp user
/// # cURL example:
///
/// ---
/// ```bash
/// curl -X POST http://192.172.0.103:4000/api/sign_up \
/// -d '{
///     "username": "root",
///     "password": "P@ssW0r3"
/// }'
/// ```
#[utoipa::path(
    post,
    path = "/api/sign_up",
    tag = "Auth",
    request_body(content = LoginData, description = "Authentication parameters", content_type = "application/json"),
    responses(
        (status = 200, description = "Current user", body = LoginAuth),
        (status = 404, description = "Some value doesn't exist"),
    )
)]
pub async fn sign_up(
    body: web::Json<LoginData>,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let body = body.into_inner();

    let user_model = users::ActiveModel {
        uuid: Set(Uuid::new_v4().to_string()),
        username: Set(body.username),
        password: Set(bcrypt::hash(body.password)?),
        max_storage: Set(209715200), // 200 MiB
        storage_usage: Set(0),
        created_at: Set(chrono::Local::now().timestamp().to_string()),
        updated_at: Set(chrono::Local::now().timestamp().to_string()),
        ..Default::default()
    };

    let user_db = user_model.insert(&data.conn).await?;

    let token = make_token(60, false);
    let expires_in = (chrono::Local::now() + Duration::hours(3)).timestamp();

    let auth_model = auth::ActiveModel {
        token: Set(token.clone()),
        user_id: Set(user_db.id),
        expires_in: Set(expires_in.to_string()),
        created_at: Set(chrono::Local::now().timestamp().to_string()),
        updated_at: Set(chrono::Local::now().timestamp().to_string()),
        ..Default::default()
    };

    auth_model.insert(&data.conn).await?;

    Ok(HttpResponse::Ok().json(LoginAuth { token }))
}

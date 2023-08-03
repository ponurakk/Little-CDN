use actix_web::{Responder, web, HttpResponse};
use chrono::Duration;
use lib::{error::AppError, AppState, entities::{auth, users}, make_token};
use pwhash::bcrypt;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, ActiveModelTrait, Set};
use serde_json::json;

use super::LoginData;

pub async fn login(
    body: web::Json<LoginData>,
    data: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let body = body.into_inner();
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(body.username))
        .one(&data.conn)
        .await?
        .ok_or(AppError::ActixError(actix_web::error::ErrorUnauthorized("Username or password are invalid")))?;

    if !bcrypt::verify(body.password, &user.password) {
        return Err(AppError::ActixError(actix_web::error::ErrorUnauthorized("Username or password are invalid")))
    }

    let auth = auth::Entity::find()
        .filter(auth::Column::UserId.eq(user.id))
        .one(&data.conn)
        .await?;

    let expires_in = (chrono::Local::now() + Duration::hours(3)).timestamp();
    let token = make_token(60, false);
    match auth {
        Some(auth_token) => {
            let mut auth_token: auth::ActiveModel = auth_token.into();

            auth_token.expires_in = Set(expires_in.to_string());
            auth_token.token = Set(token.clone());
            auth_token.updated_at = Set(chrono::Local::now().timestamp().to_string());

            auth_token.update(&data.conn).await?;
        },
        None => {
            let auth_activemodel = auth::ActiveModel {
                user_id: Set(user.id),
                token: Set(token.clone()),
                expires_in: Set(expires_in.to_string()),
                created_at: Set(chrono::Local::now().timestamp().to_string()),
                updated_at: Set(chrono::Local::now().timestamp().to_string()),
                ..Default::default()
            };

            auth_activemodel.insert(&data.conn).await?;
        }
    }

    Ok(
        HttpResponse::Ok().json(json!({
            "token": token
        }))
    )
}

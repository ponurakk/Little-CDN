use std::collections::HashMap;
use std::fs;
use actix_web::{ web, get, post, delete, HttpResponse, Error };
use serde_json::json;

use crate::AppState;
use sled;

use crate::models::user::User;

#[post("/users/{name}")]
pub async fn add_user(
    path: web::Path<String>,
    body: web::Json<User>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let users = db.open_tree(b"users").unwrap();
    let user = User {
        name: path.into_inner(),
        token: body.token.clone(),
    };
    let encoded = bincode::serialize(&user).unwrap();
    users.insert(&body.name, &*encoded).unwrap();

    let dir = format!("./{}/{}", data.files_path.display(), user.name);
    fs::create_dir_all(&dir)?;

    Ok(
        HttpResponse::Created()
            .content_type("application/json")
            .json(json!({ "message": "Created" }))
    )
}

#[get("/users")]
pub async fn get_users(
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let users = db.open_tree(b"users").unwrap();
    let mut users_map: HashMap<String, User> = HashMap::new();

    for user in users.iter() {
        let key = &user.as_ref().unwrap().0;
        let value = &user.as_ref().unwrap().1;
        let decoded_key = std::str::from_utf8(key).unwrap().to_owned();
        let decoded_value = bincode::deserialize(&value[..]).unwrap();
        users_map.insert(decoded_key, decoded_value);
    }

    Ok(
        HttpResponse::Created()
            .content_type("application/json")
            .json(json!{ users_map })
    )
}

#[get("/users/{name}")]
pub async fn get_user(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let users = db.open_tree(b"users").unwrap();
    let user = users.get(path.into_inner()).unwrap().unwrap();
    let decoded: User = bincode::deserialize(&user[..]).unwrap();
    Ok(
        HttpResponse::Created()
            .content_type("application/json")
            .json(json!{ decoded })
    )
}


#[delete("/users/{name}")]
pub async fn delete_user(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let db = sled::open(&data.conn).unwrap();
    let users = db.open_tree(b"users").unwrap();
    let user = users.remove(&path.clone()).unwrap();
    match user {
        Some(ref v) => {
            fs::remove_dir_all(format!("./{}/{}", data.files_path.display(), &path.clone())).unwrap();
            let files = db.open_tree(b"files").unwrap();
            files.remove(&path.into_inner()).unwrap();
            v
        },
        None => {
            return Ok(
                HttpResponse::NoContent().finish()
            );
        }
    };
    Ok(
        HttpResponse::Accepted()
            .content_type("application/json")
            .json(json!({ "message": "Removed" }))
    )
}

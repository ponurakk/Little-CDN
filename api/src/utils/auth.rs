use sled::Db;
use crate::models::user::User;

pub fn authorize(db: &Db, username: String, token: String) -> Result<User, ()> {
    let users_tree = db.open_tree(b"users").unwrap();
    let user = match users_tree.get(username).unwrap() {
        Some(v) => v,
        None => {
            return Err(());
        }
    };

    let user: User = bincode::deserialize(&user[..]).unwrap();
    if user.token != token {
        Err(())
    } else {
        Ok(user)
    }
}
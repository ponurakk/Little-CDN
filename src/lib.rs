pub mod websocket_server;
pub mod entities;
pub mod error;

use std::{path::{PathBuf, Path}, fs::{File, create_dir_all}, io::Write, borrow::Cow, net::Ipv4Addr};

use colored::Colorize;
use entities::users;
use error::AppError;
use log::Level;
use pwhash::bcrypt;
use rand::Rng;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set, ActiveModelTrait};
use tera::Tera;
use rust_embed::{RustEmbed, EmbeddedFile};

const DEFAULT_UUID: &str = "00000000-0000-0000-0000-000000000000";
pub const DEFAULT_TARGET: &str = "Little CDN";

#[derive(Debug, Clone)]
pub struct Config {
    pub stop_web: bool,
    pub address: Ipv4Addr,
    pub port: u16,
    pub log: u8,
    pub dir: PathBuf,
    pub disable_login: bool,
}

pub trait LogLevel {
	fn get_from_u8(&self) -> Level;
}

impl LogLevel for u8 {
	fn get_from_u8(&self) -> Level {
		match &self {
			0 => Level::Error,
			1 => Level::Warn,
			2 => Level::Info,
			3 => Level::Debug,
			_ => Level::Trace,
		}
	}
}

pub struct AppState {
    pub conn: DatabaseConnection,
    pub tera: Tera,
    pub config: Config,
}

impl AppState {
    pub fn new(conn: DatabaseConnection, tera: Tera, config: Config) -> Self {
        Self { conn, tera, config }
    }
}

pub async fn create_root_user(conn: &DatabaseConnection) -> Result<(), AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq("root"))
        .filter(users::Column::Uuid.eq(DEFAULT_UUID))
        .one(conn)
        .await?;

    if let Some(_) = user {} else {
        let root_password = make_token(15, true);
        let root_password_hash = bcrypt::hash(&root_password)?;
        let root_model = users::ActiveModel {
            uuid: Set(DEFAULT_UUID.into()),
            username: Set("root".into()),
            password: Set(root_password_hash),
            max_storage: Set(-1), // -1 means unlimited storage
            storage_usage: Set(0),
            created_at: Set(chrono::Local::now().timestamp().to_string()),
            updated_at: Set(chrono::Local::now().timestamp().to_string()),
            ..Default::default()
        };
        root_model.insert(conn).await?;

        eprintln!(r#"
╔═════════════════════════════╗
║ user: "{}"                ║
║ password: "{}" ║
╚═════════════════════════════╝
            "#, "root".yellow(), root_password.yellow());
        eprintln!("{}", "Warning! Root account should only be used in administrative purposes.
If you forget root password there is no way to recover it.
This message won't be shown again.\n".red().bold());
    }

    Ok(())
}

pub fn make_token(length: u8, use_special_chars: bool) -> String {
    let charset: &[u8] = if use_special_chars {
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()[]{};':\",./<>?`"
    } else {
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
    };
    let mut rng = rand::thread_rng();

    let token: String = (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset[idx] as char
        })
        .collect();
    token
}

pub fn load_html() -> Result<(), AppError> {
    macros::create_embed!(HTML, "debug_templates/views", "templates/views/");
    macros::create_embed!(Assets, "debug_templates/assets", "templates/assets/");
    Ok(())
}

pub mod macros {
    macro_rules! create_embed {
        ( $name:ident, $folder:expr, $prefix:expr $(, $include:expr )? ) => {
            create_dir_all($prefix)?;

            #[derive(RustEmbed)]
            #[folder = $folder]
            #[prefix = $prefix]
            $(#[include = $include])?
            struct $name;

            for file in $name::iter() {
                let index = $name::get(file.as_ref()).ok_or(AppError::NoneValue("template file"))?;
                match create_files(&file, &index) {
                    Ok(v) => v,
                    Err(e) => log::error!(target: DEFAULT_TARGET, "{} File: \"{}\" returned: {:?}, {}:{}", "Error:".red(), &file, e, file!(), line!()),
                };
            }
        };
    }
    pub(super) use create_embed;
}

fn create_files(filename: &str, index: &EmbeddedFile) -> Result<(), AppError> {
    let content = check_binary(index.data.clone())?;
    let path = Path::new(filename);
    if path.exists() {
        log::info!(target: DEFAULT_TARGET, "{} already exists", filename);
    } else {
        let mut new_file = File::create::<&str>(filename)?;
        new_file.write_all(content.as_ref())?;
        log::info!(target: DEFAULT_TARGET, "Created: {}", filename);
    }
	Ok(())
}

fn check_binary(data: Cow<'_, [u8]>) -> Result<Vec<u8>, AppError> {
    if data.is_ascii() {
		Ok(std::str::from_utf8(&data)?.as_bytes().to_vec())
    } else {
		Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose, Engine};

    #[test]
    fn binary_or_utf8() {
        // empty 1 pixel png image encrypted in base64
        let image_base64 = r#"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAABmJLR0QA/wD/AP+gvaeTAAAACXBIWXMAAC4jAAAuIwF4pT92AAAAB3RJTUUH5wcZDhEoNFbjfwAAABl0RVh0Q29tbWVudABDcmVhdGVkIHdpdGggR0lNUFeBDhcAAAALSURBVAjXY2AAAgAABQAB4iYFmwAAAABJRU5ErkJggg=="#;
        let image = general_purpose::STANDARD.decode(image_base64).expect("Error in decoding an image");

        // this shouldn't encode the image
        check_binary(image.clone().into()).expect("Failed to check if file is an binary file");

        // program returning 1 encrypted in base64
        let bin_base64 = r#"f0VMRgEAAAAAAAAAAAABAAIAAwAgAAEAIAABAAQAAACzATHAQM2AADQAIAAB"#;
        let bin = general_purpose::STANDARD.decode(bin_base64).expect("Error in decoding an binary");

        // this shouldn't encode the image
        check_binary(bin.into()).expect("Failed to check if file is an binary file");
    }
}

use lib::{entities::{users, files}, error::AppError};
use migration::async_trait::async_trait;
use sea_orm::{EntityTrait, DatabaseConnection, Set, ActiveModelTrait};
use file_format::FileFormat;

#[async_trait]
pub trait User {
    fn has_free_space(&self, file_size: i64) -> bool;
    async fn add_file(&self, db: &DatabaseConnection, file: &Vec<u8>, filename: &String, file_size: i64) -> Result<(), AppError>;
    async fn set_storage(&self, db: &DatabaseConnection, files_size: i64) -> Result<(), AppError>;
}

#[async_trait]
impl User for users::Model {
    fn has_free_space(&self, file_size: i64) -> bool {
        if self.max_storage == -1 { return true }
        if self.storage_usage + file_size <= self.max_storage { return true }
        false
    }

    async fn add_file(&self, db: &DatabaseConnection, file: &Vec<u8>, filename: &String, file_size: i64) -> Result<(), AppError> {
        let filetype = FileFormat::from_bytes(file);
        let filetype = filetype.short_name().unwrap_or(filetype.name());
        let active_file = files::ActiveModel {
            user_id: Set(self.id),
            filename: Set(filename.clone()),
            filetype: Set(filetype.to_string()),
            size: Set(file_size),
            created_at: Set(chrono::Local::now().timestamp().to_string()),
            updated_at: Set(chrono::Local::now().timestamp().to_string()),
            ..Default::default()
        };
        files::Entity::insert(active_file).exec(db).await?;
        Ok(())
    }

    async fn set_storage(&self, db: &DatabaseConnection, files_size: i64) -> Result<(), AppError> {
        let mut user: users::ActiveModel = (*self).clone().into();
        user.storage_usage = Set(self.storage_usage + files_size);

        user.update(db).await?;
        Ok(())
    }
}

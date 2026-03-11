use std::path::Path;

use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use super::error::{AppError, AppResult};
use crate::data::models::FileStorage;
use crate::data::repo as data_repo;

/// Determine file extension from a MIME type string.
fn extension_from_mime(mime_type: &str) -> &str {
    match mime_type {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        "application/pdf" => "pdf",
        _ => "bin",
    }
}

/// Persist raw bytes to disk under the file storage directory and create a
/// `file_storage` database record.
///
/// The file is stored at `{file_storage_path}/{YYYY}/{MM}/{uuid}.{ext}`.
///
/// Returns the created [`FileStorage`] record.
pub async fn save_file_to_storage(
    bytes: &[u8],
    mime_type: &str,
    file_name: &str,
    role: &str,
    data_object_id: Option<Uuid>,
    raw_input_id: Option<Uuid>,
    pool: &PgPool,
    file_storage_path: &Path,
) -> AppResult<FileStorage> {
    // Build storage path: {storage_path}/{year}/{month}/{uuid}.{ext}
    let now = chrono::Utc::now();
    let year = now.format("%Y");
    let month = now.format("%m");
    let file_uuid = Uuid::new_v4();
    let extension = extension_from_mime(mime_type);
    let stored_name = format!("{file_uuid}.{extension}");
    let relative_path = format!("{year}/{month}/{stored_name}");
    let full_dir = file_storage_path.join(format!("{year}/{month}"));
    let full_path = file_storage_path.join(&relative_path);

    // Ensure directory exists.
    fs::create_dir_all(&full_dir)
        .await
        .map_err(|e| AppError::Internal(format!("failed to create directory: {e}")))?;

    // Compute SHA-256 checksum.
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let checksum = hex::encode(hasher.finalize());

    let file_size = bytes.len() as i64;

    // Write file to disk.
    let mut file = fs::File::create(&full_path)
        .await
        .map_err(|e| AppError::Internal(format!("failed to create file: {e}")))?;
    file.write_all(bytes)
        .await
        .map_err(|e| AppError::Internal(format!("failed to write file: {e}")))?;
    file.flush()
        .await
        .map_err(|e| AppError::Internal(format!("failed to flush file: {e}")))?;

    // Create database record.
    let record = data_repo::create_file_storage(
        pool,
        data_object_id,
        raw_input_id,
        &relative_path,
        file_name,
        mime_type,
        file_size,
        &checksum,
        role,
    )
    .await
    .map_err(AppError::Database)?;

    Ok(record)
}

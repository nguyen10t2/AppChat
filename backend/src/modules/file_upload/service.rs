use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::api::error;
use crate::METRICS;
use crate::modules::file_upload::{
    model::{NewFile, UploadConfig},
    repository::FileRepository,
    schema::{FileEntity, FileUploadResponse},
};

#[derive(Clone)]
pub struct FileUploadService<R>
where
    R: FileRepository + Send + Sync,
{
    file_repo: Arc<R>,
    config: UploadConfig,
    cloudinary: Option<CloudinaryConfig>,
}

#[derive(Clone)]
struct CloudinaryConfig {
    cloud_name: String,
    api_key: String,
    api_secret: String,
}

fn parse_cloudinary_url(raw: &str) -> Option<CloudinaryConfig> {
    let parsed = url::Url::parse(raw).ok()?;

    let cloud_name = parsed.host_str()?.to_string();
    let api_key = parsed.username().to_string();
    let api_secret = parsed.password()?.to_string();

    if api_key.is_empty() || api_secret.is_empty() || cloud_name.is_empty() {
        return None;
    }

    Some(CloudinaryConfig {
        cloud_name,
        api_key,
        api_secret,
    })
}

#[derive(serde::Deserialize)]
struct CloudinaryUploadResult {
    secure_url: String,
    public_id: String,
}

#[derive(serde::Deserialize)]
struct CloudinaryDestroyResult {
    result: String,
}

impl<R> FileUploadService<R>
where
    R: FileRepository + Send + Sync,
{
    pub fn new(file_repo: Arc<R>, config: UploadConfig) -> Self {
        let cloudinary = Self::parse_cloudinary_from_env();
        Self {
            file_repo,
            config,
            cloudinary,
        }
    }

    pub fn with_defaults(file_repo: Arc<R>) -> Self {
        Self::new(file_repo, UploadConfig::default())
    }

    fn parse_cloudinary_from_env() -> Option<CloudinaryConfig> {
        let raw = std::env::var("CLOUDINARY_URL").ok()?;
        parse_cloudinary_url(&raw)
    }

    fn unix_timestamp() -> Result<i64, error::SystemError> {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| error::SystemError::internal_error("Không thể lấy timestamp hệ thống"))?
            .as_secs() as i64;

        Ok(ts)
    }

    fn sign_cloudinary(params: &str, api_secret: &str) -> String {
        use sha1::{Digest, Sha1};

        let mut hasher = Sha1::new();
        hasher.update(format!("{params}{api_secret}"));
        format!("{:x}", hasher.finalize())
    }

    async fn upload_to_cloudinary(
        &self,
        original_filename: &str,
        bytes: Vec<u8>,
        mime_type: &str,
    ) -> Result<(String, String, String), error::SystemError> {
        let cloudinary = self
            .cloudinary
            .as_ref()
            .ok_or_else(|| error::SystemError::internal_error("Cloudinary chưa được cấu hình"))?;

        let timestamp = Self::unix_timestamp()?;
        let public_id = format!("appchat/{}", Uuid::now_v7());
        let params_to_sign = format!("public_id={public_id}&timestamp={timestamp}");
        let signature = Self::sign_cloudinary(&params_to_sign, &cloudinary.api_secret);

        let part = reqwest::multipart::Part::bytes(bytes)
            .file_name(original_filename.to_string())
            .mime_str(mime_type)
            .map_err(|e| error::SystemError::bad_request(e.to_string()))?;

        let form = reqwest::multipart::Form::new()
            .part("file", part)
            .text("public_id", public_id.clone())
            .text("api_key", cloudinary.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature);

        let endpoint = format!(
            "https://api.cloudinary.com/v1_1/{}/auto/upload",
            cloudinary.cloud_name
        );

        let response = reqwest::Client::new()
            .post(endpoint)
            .multipart(form)
            .send()
            .await
            .map_err(|e| error::SystemError::internal_error(e.to_string()))?;

        if !response.status().is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Cloudinary upload thất bại".to_string());
            return Err(error::SystemError::internal_error(body));
        }

        let payload: CloudinaryUploadResult = response
            .json()
            .await
            .map_err(|e| error::SystemError::internal_error(e.to_string()))?;

        Ok((public_id, payload.secure_url, payload.public_id))
    }

    async fn delete_on_cloudinary(&self, public_id: &str) -> Result<(), error::SystemError> {
        let cloudinary = self
            .cloudinary
            .as_ref()
            .ok_or_else(|| error::SystemError::internal_error("Cloudinary chưa được cấu hình"))?;

        let timestamp = Self::unix_timestamp()?;
        let params_to_sign = format!("public_id={public_id}&timestamp={timestamp}");
        let signature = Self::sign_cloudinary(&params_to_sign, &cloudinary.api_secret);

        let client = reqwest::Client::new();

        for resource_type in ["image", "raw", "video"] {
            let endpoint = format!(
                "https://api.cloudinary.com/v1_1/{}/{}/destroy",
                cloudinary.cloud_name, resource_type
            );

            let response = client
                .post(&endpoint)
                .form(&[
                    ("public_id", public_id.to_string()),
                    ("api_key", cloudinary.api_key.clone()),
                    ("timestamp", timestamp.to_string()),
                    ("signature", signature.clone()),
                ])
                .send()
                .await
                .map_err(|e| error::SystemError::internal_error(e.to_string()))?;

            if !response.status().is_success() {
                continue;
            }

            let payload: CloudinaryDestroyResult = response
                .json()
                .await
                .map_err(|e| error::SystemError::internal_error(e.to_string()))?;

            if payload.result.eq_ignore_ascii_case("ok")
                || payload.result.eq_ignore_ascii_case("not found")
            {
                return Ok(());
            }
        }

        Err(error::SystemError::internal_error(
            "Không thể xóa file trên Cloudinary",
        ))
    }

    /// Validate file type and size
    fn validate_file(
        &self,
        _filename: &str,
        file_size: usize,
        mime_type: &str,
    ) -> Result<(), error::SystemError> {
        // Check file size
        if file_size > self.config.max_file_size {
            return Err(error::SystemError::bad_request(format!(
                "Kích thước tệp vượt quá giới hạn cho phép {} bytes",
                self.config.max_file_size
            )));
        }

        // Check MIME type
        if !self
            .config
            .allowed_mime_types
            .contains(&mime_type.to_string())
        {
            return Err(error::SystemError::bad_request(format!(
                "Loại tệp '{}' không được hỗ trợ",
                mime_type
            )));
        }

        Ok(())
    }

    /// Generate unique filename
    fn generate_filename(&self, original_filename: &str) -> String {
        let extension = Path::new(original_filename)
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        let uuid = Uuid::now_v7();
        if extension.is_empty() {
            uuid.to_string()
        } else {
            format!("{}.{}", uuid, extension)
        }
    }

    /// Save file to disk
    async fn save_file(&self, filename: &str, bytes: &[u8]) -> Result<String, error::SystemError> {
        // Create upload directory if it doesn't exist
        tokio::fs::create_dir_all(&self.config.upload_dir).await?;

        let file_path = format!("{}/{}", self.config.upload_dir, filename);
        tokio::fs::write(&file_path, bytes).await?;

        Ok(file_path)
    }

    /// Upload file and save metadata
    pub async fn upload_file(
        &self,
        original_filename: String,
        bytes: Vec<u8>,
        mime_type: String,
        uploaded_by: Uuid,
    ) -> Result<FileUploadResponse, error::SystemError> {
        METRICS.inc_upload_attempt();

        let result = self
            .upload_file_inner(original_filename, bytes, mime_type, uploaded_by)
            .await;

        if result.is_err() {
            METRICS.inc_upload_failure();
        }

        result
    }

    async fn upload_file_inner(
        &self,
        original_filename: String,
        bytes: Vec<u8>,
        mime_type: String,
        uploaded_by: Uuid,
    ) -> Result<FileUploadResponse, error::SystemError> {
        let file_size = bytes.len();

        // Validate file
        self.validate_file(&original_filename, file_size, &mime_type)?;

        // Generate unique filename
        let (filename, response_url, storage_path) = if self.cloudinary.is_some() {
            let (public_id, secure_url, returned_public_id) = self
                .upload_to_cloudinary(&original_filename, bytes, &mime_type)
                .await?;

            (
                public_id,
                secure_url,
                format!("cloudinary://{returned_public_id}"),
            )
        } else {
            let filename = self.generate_filename(&original_filename);
            let storage_path = self.save_file(&filename, &bytes).await?;
            let response_url = format!("{}/{}", self.config.base_url, filename);
            (filename, response_url, storage_path)
        };

        // Save metadata to database
        let mut tx = self.file_repo.get_pool().begin().await?;

        let new_file = NewFile {
            filename: filename.clone(),
            original_filename,
            mime_type,
            file_size: file_size as i64,
            storage_path,
            uploaded_by,
        };

        let file_entity = self.file_repo.create(&new_file, &mut *tx).await?;
        tx.commit().await?;

        // Build response
        Ok(FileUploadResponse {
            id: file_entity.id,
            filename: file_entity.filename,
            original_filename: file_entity.original_filename,
            mime_type: file_entity.mime_type,
            file_size: file_entity.file_size,
            url: response_url,
            created_at: file_entity.created_at,
        })
    }

    /// Get file metadata by ID
    pub async fn get_file(&self, file_id: &Uuid) -> Result<Option<FileEntity>, error::SystemError> {
        self.file_repo.find_by_id(file_id).await
    }

    /// Delete file
    pub async fn delete_file(&self, file_id: &Uuid) -> Result<(), error::SystemError> {
        // Get file metadata first
        let file = self
            .file_repo
            .find_by_id(file_id)
            .await?
            .ok_or_else(|| error::SystemError::not_found("Không tìm thấy tệp"))?;

        if let Some(public_id) = file.storage_path.strip_prefix("cloudinary://") {
            self.delete_on_cloudinary(public_id).await?;
        } else {
            // Delete from disk
            tokio::fs::remove_file(&file.storage_path).await.ok();
        }

        // Delete from database
        let mut tx = self.file_repo.get_pool().begin().await?;
        self.file_repo.delete(file_id, &mut *tx).await?;
        tx.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::parse_cloudinary_url;

    #[test]
    fn test_parse_cloudinary_url_valid() {
        let parsed = parse_cloudinary_url("cloudinary://api_key_123:secret_456@demo_cloud");
        assert!(parsed.is_some());

        let cfg = parsed.expect("valid cloudinary url should parse");
        assert_eq!(cfg.cloud_name, "demo_cloud");
        assert_eq!(cfg.api_key, "api_key_123");
        assert_eq!(cfg.api_secret, "secret_456");
    }

    #[test]
    fn test_parse_cloudinary_url_invalid_missing_secret() {
        let parsed = parse_cloudinary_url("cloudinary://api_key_only@demo_cloud");
        assert!(parsed.is_none());
    }

    #[test]
    fn test_parse_cloudinary_url_invalid_format() {
        let parsed = parse_cloudinary_url("not-a-url");
        assert!(parsed.is_none());
    }
}

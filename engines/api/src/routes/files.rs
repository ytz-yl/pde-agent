/// POST /files — upload a tensor file for use as history input in /solve.
///
/// Accepts multipart/form-data with a single field named "file".
/// Supported formats (by file extension): .h5 / .hdf5 / .npy / .pt / .npz
///
/// Returns:
/// ```json
/// {
///   "success": true,
///   "data": {
///     "file_id": "3f2a1b…",
///     "filename": "history.h5",
///     "format": "hdf5",
///     "size_bytes": 204800
///   }
/// }
/// ```
///
/// Files are stored under $SOLVER_UPLOAD_DIR (default: /tmp/pde-solver-uploads).
/// They are referenced in /solve via `pde.history.file_id`.
use axum::{
    extract::Multipart,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::path::PathBuf;
use tokio::fs;
use tracing::{info, warn};
use uuid::Uuid;

use crate::error::ApiError;
use crate::models::ApiResponse;

// ---------------------------------------------------------------------------
// Response type
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct FileUploadData {
    pub file_id: String,
    pub filename: String,
    pub format: String,
    pub size_bytes: usize,
    pub path: String,
}

// ---------------------------------------------------------------------------
// Upload directory helpers
// ---------------------------------------------------------------------------

pub fn upload_dir() -> PathBuf {
    std::env::var("SOLVER_UPLOAD_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/tmp/pde-solver-uploads"))
}

/// Map a file extension to a canonical format name.
fn detect_format(filename: &str) -> Option<&'static str> {
    let ext = filename.rsplit('.').next()?.to_lowercase();
    match ext.as_str() {
        "h5" | "hdf5" => Some("hdf5"),
        "npy"         => Some("npy"),
        "npz"         => Some("npz"),
        "pt" | "pth"  => Some("pt"),
        _             => None,
    }
}

/// Return the absolute path for a given file_id in the upload directory.
pub fn file_path_for_id(file_id: &str) -> Option<PathBuf> {
    let dir = upload_dir();
    // Scan for a file whose stem is file_id.
    // (We keep the original extension so the Python bridge can detect format.)
    if let Ok(mut entries) = std::fs::read_dir(&dir) {
        while let Some(Ok(entry)) = entries.next() {
            let p = entry.path();
            if p.file_stem().and_then(|s| s.to_str()) == Some(file_id) {
                return Some(p);
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub async fn upload_file(
    mut multipart: Multipart,
) -> Result<impl IntoResponse, ApiError> {
    // Ensure upload directory exists.
    let dir = upload_dir();
    fs::create_dir_all(&dir).await.map_err(|e| {
        ApiError::Internal(anyhow::anyhow!("Cannot create upload dir {}: {}", dir.display(), e))
    })?;

    // Read the first "file" field from the multipart body.
    let mut found = false;
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Multipart error: {e}"))
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        if field_name != "file" {
            continue;
        }

        let original_filename = field
            .file_name()
            .unwrap_or("upload.bin")
            .to_string();

        let format = detect_format(&original_filename).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Unsupported file format for '{}'. Allowed: .h5, .hdf5, .npy, .npz, .pt, .pth",
                original_filename
            ))
        })?;

        // Generate a unique ID and build the on-disk path (keeping extension).
        let file_id = Uuid::new_v4().to_string();
        let ext = original_filename.rsplit('.').next().unwrap_or("bin");
        let dest_path = dir.join(format!("{}.{}", file_id, ext));

        let bytes = field.bytes().await.map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to read upload bytes: {e}"))
        })?;

        let size_bytes = bytes.len();

        fs::write(&dest_path, &bytes).await.map_err(|e| {
            ApiError::Internal(anyhow::anyhow!("Failed to write upload to {}: {}", dest_path.display(), e))
        })?;

        info!(
            file_id = %file_id,
            filename = %original_filename,
            format = %format,
            size_bytes = size_bytes,
            path = %dest_path.display(),
            "File uploaded"
        );

        found = true;
        let _found = found; // suppress unused_assignments warning
        return Ok((
            StatusCode::OK,
            Json(ApiResponse::ok(FileUploadData {
                file_id,
                filename: original_filename,
                format: format.to_string(),
                size_bytes,
                path: dest_path.to_string_lossy().to_string(),
            })),
        ).into_response());
    }

    if !found {
        warn!("Upload request contained no 'file' field");
        return Err(ApiError::BadRequest(
            "Multipart body must contain a field named 'file'".to_string()
        ));
    }

    unreachable!()
}

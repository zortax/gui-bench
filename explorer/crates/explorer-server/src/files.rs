//! The files of a cell that the explorer shows as they are: the screenshots and the tail of the
//! app log. The cell comes from the run index, so a request reaches no other path.

use std::sync::Arc;

use axum::Extension;
use axum::extract::Path;
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use explorer_data::{CellFile, Store};

/// The largest log tail the explorer sends.
const LOG_TAIL: usize = 256 * 1024;

/// Serves a screenshot or `app.log` of a cell.
pub async fn serve(
    Extension(store): Extension<Arc<Store>>,
    Path((run, cell, file)): Path<(String, u32, String)>,
) -> Response {
    let Some(kind) = CellFile::parse(&file) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let found = tokio::task::spawn_blocking(move || {
        let data = store.run(&run).map_err(|e| e.to_string())?;
        let path = data.cell_file(cell, kind).map_err(|e| e.to_string())?;
        std::fs::read(path).map_err(|e| e.to_string())
    })
    .await;
    match found {
        Ok(Ok(bytes)) => match kind {
            CellFile::Screenshot(_) => (
                [
                    (header::CONTENT_TYPE, "image/png"),
                    (header::CACHE_CONTROL, "public, max-age=3600"),
                ],
                bytes,
            )
                .into_response(),
            CellFile::Log => {
                let start = bytes.len().saturating_sub(LOG_TAIL);
                let text = String::from_utf8_lossy(&bytes[start..]).into_owned();
                ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], text).into_response()
            }
        },
        Ok(Err(message)) => (StatusCode::NOT_FOUND, message).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

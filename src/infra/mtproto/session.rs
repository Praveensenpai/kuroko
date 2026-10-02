use anyhow::{Context, Result};
use grammers_session::storages::SqliteSession;
use std::path::Path;
use std::sync::Arc;

pub async fn load_or_create_session<P: AsRef<Path>>(path: P) -> Result<Arc<SqliteSession>> {
    let session = SqliteSession::open(path.as_ref()).await.with_context(|| {
        format!(
            "Failed to open session database at {}",
            path.as_ref().display()
        )
    })?;
    Ok(Arc::new(session))
}

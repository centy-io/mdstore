/// Get current timestamp in ISO 8601 format
#[must_use]
pub(crate) fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

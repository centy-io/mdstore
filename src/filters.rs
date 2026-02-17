//! Filter options for listing items.

/// Common filter options for listing items
#[derive(Debug, Clone, Default)]
pub struct Filters {
    /// Filter by status
    pub status: Option<String>,
    /// Filter by priority
    pub priority: Option<u32>,
    /// Include soft-deleted items
    pub include_deleted: bool,
    /// Limit number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

impl Filters {
    /// Create a new empty filter
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by status
    #[must_use]
    pub fn with_status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Filter by priority
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Include soft-deleted items
    #[must_use]
    pub fn include_deleted(mut self) -> Self {
        self.include_deleted = true;
        self
    }

    /// Limit results
    #[must_use]
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Offset results for pagination
    #[must_use]
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filters_default() {
        let filters = Filters::default();
        assert!(filters.status.is_none());
        assert!(filters.priority.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_filters_new() {
        let filters = Filters::new();
        assert!(filters.status.is_none());
        assert!(filters.priority.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_filters_with_status() {
        let filters = Filters::new().with_status("open");
        assert_eq!(filters.status, Some("open".to_string()));
    }

    #[test]
    fn test_filters_with_status_string() {
        let filters = Filters::new().with_status("in-progress".to_string());
        assert_eq!(filters.status, Some("in-progress".to_string()));
    }

    #[test]
    fn test_filters_with_priority() {
        let filters = Filters::new().with_priority(1);
        assert_eq!(filters.priority, Some(1));
    }

    #[test]
    fn test_filters_include_deleted() {
        let filters = Filters::new().include_deleted();
        assert!(filters.include_deleted);
    }

    #[test]
    fn test_filters_with_limit() {
        let filters = Filters::new().with_limit(10);
        assert_eq!(filters.limit, Some(10));
    }

    #[test]
    fn test_filters_with_offset() {
        let filters = Filters::new().with_offset(5);
        assert_eq!(filters.offset, Some(5));
    }

    #[test]
    fn test_filters_chained() {
        let filters = Filters::new()
            .with_status("open")
            .with_priority(2)
            .include_deleted()
            .with_limit(20)
            .with_offset(10);

        assert_eq!(filters.status, Some("open".to_string()));
        assert_eq!(filters.priority, Some(2));
        assert!(filters.include_deleted);
        assert_eq!(filters.limit, Some(20));
        assert_eq!(filters.offset, Some(10));
    }

    #[test]
    fn test_filters_clone() {
        let filters = Filters::new().with_status("open").with_priority(1);
        let cloned = filters.clone();
        assert_eq!(cloned.status, Some("open".to_string()));
        assert_eq!(cloned.priority, Some(1));
    }

    #[test]
    fn test_filters_debug() {
        let filters = Filters::new().with_status("open");
        let debug = format!("{filters:?}");
        assert!(debug.contains("Filters"));
        assert!(debug.contains("open"));
    }
}

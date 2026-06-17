//! Filter options for listing items.

/// Common filter options for listing items
#[derive(Debug, Clone, Default)]
pub struct Filters {
    /// Filter by statuses (any match); None = no filter
    pub statuses: Option<Vec<String>>,
    /// Filter by exact priority; None = no filter
    pub priority: Option<u32>,
    /// Filter by priority less than or equal to this value; None = no filter
    pub priority_lte: Option<u32>,
    /// Filter by priority greater than or equal to this value; None = no filter
    pub priority_gte: Option<u32>,
    /// Filter by tags — item must have at least one of these tags; None = no filter
    pub tags_any: Option<Vec<String>>,
    /// Filter by tags — item must have all of these tags; None = no filter
    pub tags_all: Option<Vec<String>>,
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

    /// Filter by a single status (convenience wrapper for `with_statuses`)
    #[must_use]
    pub fn with_status(self, status: impl Into<String>) -> Self {
        self.with_statuses(vec![status.into()])
    }

    /// Filter by a set of statuses (any match)
    #[must_use]
    pub fn with_statuses(mut self, statuses: Vec<String>) -> Self {
        self.statuses = Some(statuses);
        self
    }

    /// Filter by exact priority
    #[must_use]
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = Some(priority);
        self
    }

    /// Filter by priority less than or equal to this value
    #[must_use]
    pub fn with_priority_lte(mut self, priority: u32) -> Self {
        self.priority_lte = Some(priority);
        self
    }

    /// Filter by priority greater than or equal to this value
    #[must_use]
    pub fn with_priority_gte(mut self, priority: u32) -> Self {
        self.priority_gte = Some(priority);
        self
    }

    /// Filter to items that have at least one of the given tags
    #[must_use]
    pub fn with_tags_any(mut self, tags: Vec<String>) -> Self {
        self.tags_any = Some(tags);
        self
    }

    /// Filter to items that have all of the given tags
    #[must_use]
    pub fn with_tags_all(mut self, tags: Vec<String>) -> Self {
        self.tags_all = Some(tags);
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
        assert!(filters.statuses.is_none());
        assert!(filters.priority.is_none());
        assert!(filters.priority_lte.is_none());
        assert!(filters.priority_gte.is_none());
        assert!(filters.tags_any.is_none());
        assert!(filters.tags_all.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_filters_new() {
        let filters = Filters::new();
        assert!(filters.statuses.is_none());
        assert!(filters.priority.is_none());
        assert!(!filters.include_deleted);
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_filters_with_status() {
        let filters = Filters::new().with_status("open");
        assert_eq!(filters.statuses, Some(vec!["open".to_string()]));
    }

    #[test]
    fn test_filters_with_status_string() {
        let filters = Filters::new().with_status("in-progress".to_string());
        assert_eq!(filters.statuses, Some(vec!["in-progress".to_string()]));
    }

    #[test]
    fn test_filters_with_statuses() {
        let filters =
            Filters::new().with_statuses(vec!["open".to_string(), "in-progress".to_string()]);
        assert_eq!(
            filters.statuses,
            Some(vec!["open".to_string(), "in-progress".to_string()])
        );
    }

    #[test]
    fn test_filters_with_priority() {
        let filters = Filters::new().with_priority(1);
        assert_eq!(filters.priority, Some(1));
    }

    #[test]
    fn test_filters_with_priority_lte() {
        let filters = Filters::new().with_priority_lte(2);
        assert_eq!(filters.priority_lte, Some(2));
    }

    #[test]
    fn test_filters_with_priority_gte() {
        let filters = Filters::new().with_priority_gte(1);
        assert_eq!(filters.priority_gte, Some(1));
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

        assert_eq!(filters.statuses, Some(vec!["open".to_string()]));
        assert_eq!(filters.priority, Some(2));
        assert!(filters.include_deleted);
        assert_eq!(filters.limit, Some(20));
        assert_eq!(filters.offset, Some(10));
    }

    #[test]
    fn test_filters_clone() {
        let filters = Filters::new().with_status("open").with_priority(1);
        let cloned = filters.clone();
        assert_eq!(cloned.statuses, Some(vec!["open".to_string()]));
        assert_eq!(cloned.priority, Some(1));
        // The clone must equal the original it was derived from.
        assert_eq!(cloned.statuses, filters.statuses);
        assert_eq!(cloned.priority, filters.priority);
    }

    #[test]
    fn test_filters_debug() {
        let filters = Filters::new().with_status("open");
        let debug = format!("{filters:?}");
        assert!(debug.contains("Filters"));
        assert!(debug.contains("open"));
    }

    #[test]
    fn test_filters_with_tags_any() {
        let filters = Filters::new().with_tags_any(vec!["bug".to_string(), "frontend".to_string()]);
        assert_eq!(
            filters.tags_any,
            Some(vec!["bug".to_string(), "frontend".to_string()])
        );
        assert!(filters.tags_all.is_none());
    }

    #[test]
    fn test_filters_with_tags_all() {
        let filters = Filters::new().with_tags_all(vec!["bug".to_string(), "urgent".to_string()]);
        assert_eq!(
            filters.tags_all,
            Some(vec!["bug".to_string(), "urgent".to_string()])
        );
        assert!(filters.tags_any.is_none());
    }
}

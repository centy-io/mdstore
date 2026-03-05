//! Display number reconciliation for resolving conflicts.
//!
//! When multiple users create items offline, they may assign the same display
//! number. This module detects and resolves such conflicts by:
//! 1. Keeping the oldest item's display number (by `created_at`)
//! 2. Reassigning newer items to the next available number

use crate::error::StoreError;
use crate::frontmatter::{generate_frontmatter, parse_frontmatter};
use crate::types::Frontmatter;
use crate::util::now_iso;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex};
use tokio::fs;
use tokio::sync::{Mutex as TokioMutex, OwnedMutexGuard};

/// Per-directory locks for serializing display number assignment within a process.
static DIR_LOCKS: LazyLock<Mutex<HashMap<PathBuf, Arc<TokioMutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn dir_lock(type_dir: &Path) -> Arc<TokioMutex<()>> {
    let canonical = std::fs::canonicalize(type_dir).unwrap_or_else(|_| type_dir.to_path_buf());
    let mut map = DIR_LOCKS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Arc::clone(
        map.entry(canonical)
            .or_insert_with(|| Arc::new(TokioMutex::new(()))),
    )
}

/// Acquire the per-directory lock for display number assignment.
///
/// The returned guard must be held until the item file has been written.
/// This prevents concurrent creates from assigning the same display number.
pub(crate) async fn acquire_display_number_lock(type_dir: &Path) -> OwnedMutexGuard<()> {
    dir_lock(type_dir).lock_owned().await
}

/// Information about an item needed for reconciliation.
#[derive(Debug, Clone)]
struct ItemInfo {
    /// Item ID (from filename without .md)
    id: String,
    display_number: u32,
    /// Parsed creation timestamp for reliable chronological ordering.
    ///
    /// Stored as `DateTime<Utc>` rather than the raw string so that sorting is
    /// always correct regardless of the timezone offset present in the value.
    /// String-based comparison of ISO 8601 timestamps only coincides with
    /// chronological order when every timestamp uses the same UTC offset; a
    /// value like `"2024-01-01T10:00:00-02:00"` sorts *before*
    /// `"2024-01-01T10:00:00Z"` lexicographically even though it represents a
    /// *later* point in time.
    created_at: DateTime<Utc>,
}

/// Check if a filename is a valid item `.md` file (not `config.yaml` or other files).
fn is_item_file(name: &str) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

/// Get the next available display number for an item type.
///
/// Scans all `*.md` files in the type storage directory and returns max + 1.
pub async fn get_next_display_number(type_dir: &Path) -> Result<u32, StoreError> {
    if !type_dir.exists() {
        return Ok(1);
    }

    let mut max_number: u32 = 0;
    let mut entries = fs::read_dir(type_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !is_item_file(&name) {
            continue;
        }

        if let Ok(content) = fs::read_to_string(entry.path()).await {
            if let Ok((frontmatter, _, _)) = parse_frontmatter::<Frontmatter>(&content) {
                if let Some(dn) = frontmatter.display_number {
                    max_number = max_number.max(dn);
                }
            }
        }
    }

    Ok(max_number.saturating_add(1))
}

/// Reconcile display numbers to resolve conflicts in an item type.
///
/// Scans all items, finds duplicate display numbers, and reassigns them so
/// each item has a unique display number. The oldest item (by `created_at`)
/// keeps its original number.
///
/// Returns the number of items that were reassigned.
pub async fn reconcile_display_numbers(type_dir: &Path) -> Result<u32, StoreError> {
    if !type_dir.exists() {
        return Ok(0);
    }

    // Step 1: Read all items and their display numbers
    let mut items: Vec<ItemInfo> = Vec::new();
    let mut entries = fs::read_dir(type_dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        if !entry.file_type().await?.is_file() {
            continue;
        }

        let name = match entry.file_name().to_str() {
            Some(n) => n.to_string(),
            None => continue,
        };

        if !is_item_file(&name) {
            continue;
        }

        let content = match fs::read_to_string(entry.path()).await {
            Ok(c) => c,
            Err(_) => continue,
        };

        let frontmatter = match parse_frontmatter::<Frontmatter>(&content) {
            Ok((fm, _, _)) => fm,
            Err(_) => continue,
        };

        if let Some(dn) = frontmatter.display_number {
            let item_id = name.trim_end_matches(".md").to_string();
            let created_at_str = &frontmatter.created_at;
            let created_at = DateTime::parse_from_rfc3339(created_at_str)
                .map(|dt| dt.to_utc())
                .map_err(|_| {
                    StoreError::Custom(format!(
                        "Invalid created_at timestamp '{created_at_str}' in item '{item_id}'"
                    ))
                })?;
            items.push(ItemInfo {
                id: item_id,
                display_number: dn,
                created_at,
            });
        }
    }

    // Step 2: Find duplicates (group by display_number)
    let mut by_display_number: HashMap<u32, Vec<&ItemInfo>> = HashMap::new();
    for item in &items {
        by_display_number
            .entry(item.display_number)
            .or_default()
            .push(item);
    }

    // Step 3: Find max display number for reassignment
    let max_display_number = items.iter().map(|i| i.display_number).max().unwrap_or(0);

    // Step 4: Process duplicates
    let mut reassignments: Vec<(ItemInfo, u32)> = Vec::new();
    let mut next_available = max_display_number.saturating_add(1);

    for (display_number, mut group) in by_display_number {
        if group.len() <= 1 {
            continue;
        }

        // Skip display_number 0 (items without display numbers)
        if display_number == 0 {
            for item in &group {
                reassignments.push(((*item).clone(), next_available));
                next_available = next_available.saturating_add(1);
            }
            continue;
        }

        // Sort by created_at (oldest first). We use the pre-parsed DateTime<Utc>
        // value so that items with different timezone offsets are ordered
        // correctly; lexicographic string comparison is not equivalent to
        // chronological order when offsets differ.
        group.sort_by_key(|item| item.created_at);

        // Keep the first (oldest), reassign the rest
        for item in group.iter().skip(1) {
            reassignments.push(((*item).clone(), next_available));
            next_available = next_available.saturating_add(1);
        }
    }

    // Step 5: Write reassignments
    let reassignment_count = reassignments.len() as u32;

    for (item_info, new_display_number) in reassignments {
        let file_path = type_dir.join(format!("{}.md", item_info.id));
        let content = fs::read_to_string(&file_path).await?;
        let (mut frontmatter, title, body): (Frontmatter, String, String) =
            parse_frontmatter(&content)
                .map_err(|e| StoreError::Custom(format!("Frontmatter error: {e}")))?;
        frontmatter.display_number = Some(new_display_number);
        frontmatter.updated_at = now_iso();
        let comment = crate::frontmatter::extract_frontmatter_comment(&content);
        let new_content = generate_frontmatter(&frontmatter, &title, &body, comment.as_deref());
        fs::write(&file_path, new_content).await?;
    }

    Ok(reassignment_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontmatter::generate_frontmatter;
    use tempfile::TempDir;

    async fn create_test_item(
        storage_path: &Path,
        id: &str,
        display_number: u32,
        created_at: &str,
    ) {
        let frontmatter = Frontmatter {
            display_number: Some(display_number),
            status: Some("open".to_string()),
            priority: Some(2),
            created_at: created_at.to_string(),
            updated_at: created_at.to_string(),
            deleted_at: None,
            custom_fields: HashMap::new(),
        };

        let content = generate_frontmatter(&frontmatter, &format!("Item {id}"), "", None);
        fs::write(storage_path.join(format!("{id}.md")), content)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_get_next_display_number_empty() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        let next = get_next_display_number(&storage_path).await.unwrap();
        assert_eq!(next, 1);
    }

    #[tokio::test]
    async fn test_get_next_display_number_with_existing() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        create_test_item(&storage_path, "item-1", 5, "2024-01-01T10:00:00Z").await;

        let next = get_next_display_number(&storage_path).await.unwrap();
        assert_eq!(next, 6);
    }

    #[tokio::test]
    async fn test_reconcile_no_conflicts() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        create_test_item(&storage_path, "item-1", 1, "2024-01-01T10:00:00Z").await;
        create_test_item(&storage_path, "item-2", 2, "2024-01-01T11:00:00Z").await;

        let reassigned = reconcile_display_numbers(&storage_path).await.unwrap();
        assert_eq!(reassigned, 0);
    }

    #[tokio::test]
    async fn test_reconcile_with_conflict() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        // Both have display_number 4, but different created_at
        create_test_item(&storage_path, "item-1", 4, "2024-01-01T10:00:00Z").await;
        create_test_item(&storage_path, "item-2", 4, "2024-01-01T10:05:00Z").await;
        create_test_item(&storage_path, "item-3", 5, "2024-01-01T10:10:00Z").await;

        let reassigned = reconcile_display_numbers(&storage_path).await.unwrap();
        assert_eq!(reassigned, 1);

        // Verify the older one kept display_number 4
        let content = fs::read_to_string(storage_path.join("item-1.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<Frontmatter>(&content).unwrap();
        assert_eq!(fm.display_number, Some(4));

        // Verify the newer one was reassigned to 6 (max was 5, so next is 6)
        let content = fs::read_to_string(storage_path.join("item-2.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<Frontmatter>(&content).unwrap();
        assert_eq!(fm.display_number, Some(6));
    }

    #[tokio::test]
    async fn test_reconcile_empty_directory() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");

        let reassigned = reconcile_display_numbers(&storage_path).await.unwrap();
        assert_eq!(reassigned, 0);
    }

    /// Verify that conflict resolution uses true chronological order even when
    /// timestamps carry non-UTC offsets.
    ///
    /// `"2024-01-01T10:00:00-02:00"` represents 12:00 UTC and is therefore
    /// *newer* than `"2024-01-01T10:00:00Z"` (10:00 UTC), yet sorts *before*
    /// it lexicographically (because `-` < `Z` in ASCII).  String-based sorting
    /// would incorrectly treat the `-02:00` item as older and keep its display
    /// number; DateTime-based sorting correctly identifies the UTC item as older.
    #[tokio::test]
    async fn test_reconcile_respects_timezone_offsets() {
        let temp = TempDir::new().unwrap();
        let storage_path = temp.path().join("items");
        fs::create_dir_all(&storage_path).await.unwrap();

        // item-1: 10:00 UTC — the genuinely older item; should keep display #1
        create_test_item(&storage_path, "item-1", 1, "2024-01-01T10:00:00Z").await;
        // item-2: 12:00 UTC (expressed as 10:00-02:00) — the newer item;
        // sorts *before* item-1 lexicographically, but is actually later.
        create_test_item(&storage_path, "item-2", 1, "2024-01-01T10:00:00-02:00").await;

        let reassigned = reconcile_display_numbers(&storage_path).await.unwrap();
        assert_eq!(reassigned, 1);

        // item-1 (older in UTC) must retain display number 1
        let content = fs::read_to_string(storage_path.join("item-1.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<Frontmatter>(&content).unwrap();
        assert_eq!(fm.display_number, Some(1));

        // item-2 (newer in UTC) must be reassigned
        let content = fs::read_to_string(storage_path.join("item-2.md"))
            .await
            .unwrap();
        let (fm, _, _) = parse_frontmatter::<Frontmatter>(&content).unwrap();
        assert_ne!(fm.display_number, Some(1));
    }
}

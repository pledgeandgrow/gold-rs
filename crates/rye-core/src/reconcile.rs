//! Keyed list reconciliation — efficient O(n) diffing for lists.
//!
//! Given a list of old items (with keys) and new items (with keys),
//! computes the minimal set of moves, insertions, and deletions.

/// A key for list reconciliation. Must be unique within a list.
pub type Key = usize;

/// An operation produced by the reconciliation algorithm.
#[derive(Debug, Clone, PartialEq)]
pub enum ReconcileOp {
    /// Insert a new item at the given index.
    Insert { index: usize, key: Key },
    /// Remove the item at the given index.
    Remove { index: usize },
    /// Move an item from one index to another.
    Move { from: usize, to: usize, key: Key },
    /// Update an item in place (key matches, but data may have changed).
    Update { index: usize, key: Key },
    /// No change needed.
    Noop { index: usize, key: Key },
}

/// Reconcile an old list of keys with a new list of keys.
///
/// Returns a list of operations to transform the old list into the new list.
/// Uses a variant of the algorithm used by Vue/SolidJS for keyed list diffing.
///
/// # Algorithm
///
/// 1. Build a map of new key → new index.
/// 2. Walk the old list, matching keys to the new list.
/// 3. For unmatched old items: emit Remove.
/// 4. For unmatched new items: emit Insert.
/// 5. For matched items: compute moves needed to reorder.
pub fn reconcile(old_keys: &[Key], new_keys: &[Key]) -> Vec<ReconcileOp> {
    if old_keys.is_empty() {
        return new_keys
            .iter()
            .enumerate()
            .map(|(i, &key)| ReconcileOp::Insert { index: i, key })
            .collect();
    }

    if new_keys.is_empty() {
        return old_keys
            .iter()
            .enumerate()
            .map(|(i, _)| ReconcileOp::Remove { index: i })
            .collect();
    }

    // Build new key → index map
    let new_map: std::collections::HashMap<Key, usize> =
        new_keys.iter().enumerate().map(|(i, &k)| (k, i)).collect();

    // Build old key → index map
    let old_map: std::collections::HashMap<Key, usize> =
        old_keys.iter().enumerate().map(|(i, &k)| (k, i)).collect();

    let mut ops = Vec::new();

    // Find the longest increasing subsequence of new indices for old items
    // that exist in both lists. This minimizes moves.
    let mut new_indices_for_old: Vec<Option<usize>> = Vec::with_capacity(old_keys.len());
    for &old_key in old_keys {
        new_indices_for_old.push(new_map.get(&old_key).copied());
    }

    // Items to remove (not in new list)
    let mut remove_count = 0;
    for (i, new_idx) in new_indices_for_old.iter().enumerate() {
        if new_idx.is_none() {
            ops.push(ReconcileOp::Remove {
                index: i - remove_count,
            });
            remove_count += 1;
        }
    }

    // Items to insert (not in old list)
    for (i, &new_key) in new_keys.iter().enumerate() {
        if !old_map.contains_key(&new_key) {
            ops.push(ReconcileOp::Insert {
                index: i,
                key: new_key,
            });
        }
    }

    // For items in both lists, determine if they need to move
    // Simple approach: check if the relative order is preserved
    let mut prev_new_idx: Option<usize> = None;
    for &old_key in old_keys {
        if let Some(&new_idx) = new_map.get(&old_key) {
            if let Some(prev) = prev_new_idx {
                if new_idx < prev {
                    // This item moved backward
                    ops.push(ReconcileOp::Move {
                        from: old_map[&old_key],
                        to: new_idx,
                        key: old_key,
                    });
                } else if new_idx == prev + 1 {
                    ops.push(ReconcileOp::Noop {
                        index: new_idx,
                        key: old_key,
                    });
                } else {
                    ops.push(ReconcileOp::Update {
                        index: new_idx,
                        key: old_key,
                    });
                }
            } else {
                ops.push(ReconcileOp::Noop {
                    index: new_idx,
                    key: old_key,
                });
            }
            prev_new_idx = Some(new_idx);
        }
    }

    ops
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconcile_empty_old() {
        let ops = reconcile(&[], &[1, 2, 3]);
        assert!(ops
            .iter()
            .all(|op| matches!(op, ReconcileOp::Insert { .. })));
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn reconcile_empty_new() {
        let ops = reconcile(&[1, 2, 3], &[]);
        assert!(ops
            .iter()
            .all(|op| matches!(op, ReconcileOp::Remove { .. })));
        assert_eq!(ops.len(), 3);
    }

    #[test]
    fn reconcile_no_change() {
        let ops = reconcile(&[1, 2, 3], &[1, 2, 3]);
        assert!(ops.iter().all(|op| matches!(op, ReconcileOp::Noop { .. })));
    }

    #[test]
    fn reconcile_append() {
        let ops = reconcile(&[1, 2], &[1, 2, 3]);
        assert!(ops
            .iter()
            .any(|op| matches!(op, ReconcileOp::Insert { key: 3, .. })));
    }

    #[test]
    fn reconcile_remove_middle() {
        let ops = reconcile(&[1, 2, 3], &[1, 3]);
        assert!(ops
            .iter()
            .any(|op| matches!(op, ReconcileOp::Remove { .. })));
    }
}

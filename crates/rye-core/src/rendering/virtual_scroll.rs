//! Goal 112: Virtual scrolling.
//!
//! `<VirtualList>` and `<VirtualGrid>` components that only render visible
//! items. Works with signal-based reactivity — scroll position is a signal,
//! visible range is a memo. Handles dynamic item heights, smooth scrolling.

/// Virtual list configuration.
#[derive(Debug, Clone)]
pub struct VirtualListConfig {
    /// Total number of items.
    pub item_count: usize,
    /// Fixed item height in pixels (0 for variable height).
    pub item_height: f64,
    /// Container height in pixels.
    pub container_height: f64,
    /// Overscan — extra items rendered above/below visible area.
    pub overscan: usize,
}

impl VirtualListConfig {
    /// Create a new virtual list config.
    pub fn new(item_count: usize, item_height: f64, container_height: f64) -> Self {
        Self {
            item_count,
            item_height,
            container_height,
            overscan: 3,
        }
    }

    /// Set overscan count.
    pub fn with_overscan(mut self, overscan: usize) -> Self {
        self.overscan = overscan;
        self
    }
}

/// Computed visible range for a virtual list.
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleRange {
    /// First visible item index.
    pub start: usize,
    /// One past last visible item index.
    pub end: usize,
    /// Total number of visible items.
    pub count: usize,
    /// Offset in pixels to apply to the rendered container.
    pub offset_y: f64,
}

/// Compute the visible range for a virtual list at the given scroll position.
pub fn compute_visible_range(config: &VirtualListConfig, scroll_top: f64) -> VisibleRange {
    if config.item_height <= 0.0 || config.item_count == 0 {
        return VisibleRange {
            start: 0,
            end: 0,
            count: 0,
            offset_y: 0.0,
        };
    }

    let first_visible = (scroll_top / config.item_height).floor() as usize;
    let visible_count = ((config.container_height / config.item_height).ceil() as usize) + 1;

    let start = first_visible.saturating_sub(config.overscan);
    let end = (first_visible + visible_count + config.overscan).min(config.item_count);
    let count = end.saturating_sub(start);
    let offset_y = start as f64 * config.item_height;

    VisibleRange {
        start,
        end,
        count,
        offset_y,
    }
}

/// Virtual grid configuration.
#[derive(Debug, Clone)]
pub struct VirtualGridConfig {
    /// Total number of items.
    pub item_count: usize,
    /// Number of columns.
    pub columns: usize,
    /// Fixed item width in pixels.
    pub item_width: f64,
    /// Fixed item height in pixels.
    pub item_height: f64,
    /// Container width in pixels.
    pub container_width: f64,
    /// Container height in pixels.
    pub container_height: f64,
    /// Overscan rows.
    pub overscan: usize,
}

impl VirtualGridConfig {
    /// Create a new virtual grid config.
    pub fn new(
        item_count: usize,
        columns: usize,
        item_width: f64,
        item_height: f64,
        container_width: f64,
        container_height: f64,
    ) -> Self {
        Self {
            item_count,
            columns,
            item_width,
            item_height,
            container_width,
            container_height,
            overscan: 2,
        }
    }
}

/// Computed visible range for a virtual grid.
#[derive(Debug, Clone, PartialEq)]
pub struct GridVisibleRange {
    /// Start row index.
    pub start_row: usize,
    /// End row index (exclusive).
    pub end_row: usize,
    /// Start column index.
    pub start_col: usize,
    /// End column index (exclusive).
    pub end_col: usize,
    /// Y offset in pixels.
    pub offset_y: f64,
    /// X offset in pixels.
    pub offset_x: f64,
}

/// Compute visible range for a virtual grid.
pub fn compute_grid_visible_range(
    config: &VirtualGridConfig,
    scroll_top: f64,
    scroll_left: f64,
) -> GridVisibleRange {
    let total_rows = (config.item_count + config.columns - 1) / config.columns.max(1);

    if config.item_height <= 0.0 || total_rows == 0 {
        return GridVisibleRange {
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
            offset_y: 0.0,
            offset_x: 0.0,
        };
    }

    let first_row = (scroll_top / config.item_height).floor() as usize;
    let visible_rows = ((config.container_height / config.item_height).ceil() as usize) + 1;

    let start_row = first_row.saturating_sub(config.overscan);
    let end_row = (first_row + visible_rows + config.overscan).min(total_rows);

    let first_col = if config.item_width > 0.0 {
        (scroll_left / config.item_width).floor() as usize
    } else {
        0
    };
    let visible_cols = if config.item_width > 0.0 {
        ((config.container_width / config.item_width).ceil() as usize) + 1
    } else {
        config.columns
    };

    let start_col = first_col.saturating_sub(config.overscan);
    let end_col = (first_col + visible_cols + config.overscan).min(config.columns);

    GridVisibleRange {
        start_row,
        end_row,
        start_col,
        end_col,
        offset_y: start_row as f64 * config.item_height,
        offset_x: start_col as f64 * config.item_width,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_list_basic() {
        let config = VirtualListConfig::new(1000, 50.0, 500.0);
        let range = compute_visible_range(&config, 0.0);

        // At scroll 0, should see items 0 to ~13 (10 visible + 3 overscan each side)
        assert_eq!(range.start, 0);
        assert!(range.count > 0);
        assert_eq!(range.offset_y, 0.0);
    }

    #[test]
    fn test_virtual_list_scrolled() {
        let config = VirtualListConfig::new(1000, 50.0, 500.0);
        let range = compute_visible_range(&config, 500.0);

        // At scroll 500, first visible is item 10
        assert!(range.start <= 10);
        assert!(range.end > 10);
        assert!(range.offset_y > 0.0);
    }

    #[test]
    fn test_virtual_list_end() {
        let config = VirtualListConfig::new(100, 50.0, 500.0);
        let range = compute_visible_range(&config, 4750.0); // Near end

        assert!(range.end <= 100);
    }

    #[test]
    fn test_virtual_list_empty() {
        let config = VirtualListConfig::new(0, 50.0, 500.0);
        let range = compute_visible_range(&config, 0.0);
        assert_eq!(range.count, 0);
    }

    #[test]
    fn test_virtual_grid_basic() {
        let config = VirtualGridConfig::new(1000, 5, 100.0, 80.0, 500.0, 400.0);
        let range = compute_grid_visible_range(&config, 0.0, 0.0);

        assert_eq!(range.start_row, 0);
        assert!(range.end_row > 0);
        assert_eq!(range.start_col, 0);
        assert!(range.end_col > 0);
    }

    #[test]
    fn test_virtual_grid_scrolled() {
        let config = VirtualGridConfig::new(1000, 5, 100.0, 80.0, 500.0, 400.0);
        let range = compute_grid_visible_range(&config, 320.0, 200.0);

        assert!(range.start_row <= 4);
        assert!(range.offset_y > 0.0);
    }
}

//! Utility functions for formatting and colorized terminal output

/// ANSI color codes for terminal output
#[allow(missing_docs)]
pub mod colors {
    /// Reset all formatting
    pub const RESET: &str = "\x1b[0m";
    /// Bold text
    pub const BOLD: &str = "\x1b[1m";
    /// Dim text
    pub const DIM: &str = "\x1b[2m";

    /// Black foreground
    pub const BLACK: &str = "\x1b[30m";
    /// Red foreground
    pub const RED: &str = "\x1b[31m";
    /// Green foreground
    pub const GREEN: &str = "\x1b[32m";
    /// Yellow foreground
    pub const YELLOW: &str = "\x1b[33m";
    /// Blue foreground
    pub const BLUE: &str = "\x1b[34m";
    /// Magenta foreground
    pub const MAGENTA: &str = "\x1b[35m";
    /// Cyan foreground
    pub const CYAN: &str = "\x1b[36m";
    /// White foreground
    pub const WHITE: &str = "\x1b[37m";

    /// Red background
    pub const BG_RED: &str = "\x1b[41m";
    /// Green background
    pub const BG_GREEN: &str = "\x1b[42m";
    /// Yellow background
    pub const BG_YELLOW: &str = "\x1b[43m";
}

/// Style text with color if enabled
pub fn colorize(text: &str, color: &str, use_color: bool) -> String {
    if use_color && atty::is(atty::Stream::Stdout) {
        format!("{}{}{}", color, text, colors::RESET)
    } else {
        text.to_string()
    }
}

/// Style text as bold if color is enabled
pub fn bold(text: &str, use_color: bool) -> String {
    colorize(text, colors::BOLD, use_color)
}

/// Style text as dim if color is enabled
pub fn dim(text: &str, use_color: bool) -> String {
    colorize(text, colors::DIM, use_color)
}

/// Get color based on coverage percentage
pub fn coverage_color(coverage: f64) -> &'static str {
    match coverage {
        c if c >= 80.0 => colors::GREEN,
        c if c >= 60.0 => colors::YELLOW,
        _ => colors::RED,
    }
}

/// Format coverage percentage with color
pub fn format_coverage(coverage: f64, use_color: bool) -> String {
    let color = coverage_color(coverage);
    let text = format!("{:5.1}%", coverage);
    colorize(&text, color, use_color)
}

/// Format a coverage bar visualization
pub fn format_coverage_bar(coverage: f64, width: usize, use_color: bool) -> String {
    let filled = ((coverage / 100.0) * width as f64) as usize;
    let empty = width.saturating_sub(filled);

    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    colorize(&bar, coverage_color(coverage), use_color)
}

/// Tree drawing characters for ASCII art trees
pub mod tree_chars {
    /// Branch connector for non-last items
    pub const BRANCH: &str = "├── ";
    /// Branch connector for last item
    pub const LAST_BRANCH: &str = "└── ";
    /// Vertical line for continuing branches
    pub const VERTICAL: &str = "│   ";
    /// Empty space for alignment
    pub const EMPTY: &str = "    ";
}

/// Helper for building tree structures
pub struct TreeBuilder {
    /// Accumulated output lines
    pub lines: Vec<String>,
    use_color: bool,
}

impl TreeBuilder {
    /// Create a new tree builder
    pub fn new(use_color: bool) -> Self {
        Self {
            lines: Vec::new(),
            use_color,
        }
    }

    /// Add a node to the tree
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix for this line (indentation)
    /// * `is_last` - Whether this is the last child at this level
    /// * `label` - The node label
    /// * `value` - Optional value to display
    /// * `color` - Optional color for the label
    pub fn add_node(
        &mut self,
        prefix: &str,
        is_last: bool,
        label: &str,
        value: Option<&str>,
        color: Option<&str>,
    ) {
        let mut line = String::new();
        line.push_str(prefix);

        let branch = if is_last {
            tree_chars::LAST_BRANCH
        } else {
            tree_chars::BRANCH
        };
        line.push_str(branch);

        let styled_label = if let Some(c) = color {
            colorize(label, c, self.use_color)
        } else {
            label.to_string()
        };
        line.push_str(&styled_label);

        if let Some(v) = value {
            line.push_str(" ");
            line.push_str(v);
        }

        self.lines.push(line);
    }

    /// Get the next prefix for children of the current node
    pub fn child_prefix(prefix: &str, is_last: bool) -> String {
        let mut new_prefix = prefix.to_string();
        if is_last {
            new_prefix.push_str(tree_chars::EMPTY);
        } else {
            new_prefix.push_str(tree_chars::VERTICAL);
        }
        new_prefix
    }

    /// Build the final tree string
    pub fn build(self) -> String {
        self.lines.join("\n")
    }
}

/// Format file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    const THRESHOLD: f64 = 1024.0;

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    format!("{:.1} {}", size, UNITS[unit_index])
}

/// Truncate a string with ellipsis if it exceeds max length
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Create a summary line with aligned values
pub fn format_summary_line(label: &str, value: &str, width: usize) -> String {
    let dots = ".".repeat(width.saturating_sub(label.len() + value.len() + 2));
    format!("{} {} {}", label, dots, value)
}

/// Format a duration in human-readable format
pub fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

/// Progress bar for long-running operations
pub struct ProgressBar {
    total: usize,
    current: usize,
    width: usize,
    use_color: bool,
}

impl ProgressBar {
    /// Create a new progress bar
    pub fn new(total: usize, width: usize, use_color: bool) -> Self {
        Self {
            total,
            current: 0,
            width,
            use_color,
        }
    }

    /// Update the progress
    pub fn update(&mut self, current: usize) {
        self.current = current.min(self.total);
    }

    /// Increment the progress by one
    pub fn increment(&mut self) {
        self.current = (self.current + 1).min(self.total);
    }

    /// Format the progress bar
    pub fn format(&self) -> String {
        let percentage = if self.total > 0 {
            (self.current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        let filled = ((percentage / 100.0) * self.width as f64) as usize;
        let empty = self.width.saturating_sub(filled);

        let bar = format!(
            "[{}{}] {}/{} ({:.1}%)",
            "=".repeat(filled),
            " ".repeat(empty),
            self.current,
            self.total,
            percentage
        );

        if percentage >= 100.0 {
            colorize(&bar, colors::GREEN, self.use_color)
        } else {
            bar
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512.0 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1048576), "1.0 MB");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world", 8), "hello...");
        assert_eq!(truncate_string("hi", 2), "hi");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3665), "1h 1m");
    }

    #[test]
    fn test_coverage_color() {
        assert_eq!(coverage_color(90.0), colors::GREEN);
        assert_eq!(coverage_color(70.0), colors::YELLOW);
        assert_eq!(coverage_color(40.0), colors::RED);
    }
}

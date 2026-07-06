use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Greedy word-wrap on display width (CJK-aware). Words wider than `width`
/// are hard-broken. Always returns at least one line.
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    let mut line_w = 0usize;

    for word in text.split_whitespace() {
        let word_w = word.width();
        let sep_w = if line_w == 0 { 0 } else { 1 };

        if line_w + sep_w + word_w <= width {
            if sep_w == 1 {
                line.push(' ');
            }
            line.push_str(word);
            line_w += sep_w + word_w;
            continue;
        }

        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
            line_w = 0;
        }

        if word_w <= width {
            line.push_str(word);
            line_w = word_w;
        } else {
            // Hard-break an over-long word (URLs, CJK runs without spaces).
            for ch in word.chars() {
                let cw = ch.width().unwrap_or(0);
                if line_w + cw > width && line_w > 0 {
                    lines.push(std::mem::take(&mut line));
                    line_w = 0;
                }
                line.push(ch);
                line_w += cw;
            }
        }
    }

    if !line.is_empty() || lines.is_empty() {
        lines.push(line);
    }
    lines
}

/// Number of rows `text` occupies when wrapped at `width`.
pub fn wrapped_height(text: &str, width: usize) -> usize {
    wrap_text(text, width).len()
}

/// Incremental greedy wrapper over a word stream, using the same rules as
/// `wrap_text`. Lets callers ask "which row does this point land on?" while
/// feeding a continuous flow of text (the paragraph reading view).
pub struct RowTracker {
    width: usize,
    row: usize,
    line_w: usize,
}

impl RowTracker {
    pub fn new(width: usize) -> Self {
        Self { width: width.max(1), row: 0, line_w: 0 }
    }

    /// 0-based row the next word would start on.
    pub fn row(&self) -> usize {
        self.row
    }

    /// Total rows occupied so far (at least 1).
    pub fn total_rows(&self) -> usize {
        self.row + 1
    }

    pub fn push_text(&mut self, text: &str) {
        for word in text.split_whitespace() {
            let word_w = word.width();
            let sep_w = if self.line_w == 0 { 0 } else { 1 };

            if self.line_w + sep_w + word_w <= self.width {
                self.line_w += sep_w + word_w;
                continue;
            }

            if word_w <= self.width {
                self.row += 1;
                self.line_w = word_w;
                continue;
            }

            // Hard-break an over-long word.
            if self.line_w > 0 {
                self.row += 1;
                self.line_w = 0;
            }
            for ch in word.chars() {
                let cw = ch.width().unwrap_or(0);
                if self.line_w + cw > self.width && self.line_w > 0 {
                    self.row += 1;
                    self.line_w = 0;
                }
                self.line_w += cw;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_text_single_line() {
        assert_eq!(wrap_text("hello world", 20), vec!["hello world"]);
    }

    #[test]
    fn wraps_at_word_boundary() {
        assert_eq!(
            wrap_text("For God so loved the world", 10),
            vec!["For God so", "loved the", "world"]
        );
    }

    #[test]
    fn hard_breaks_long_word() {
        assert_eq!(wrap_text("abcdefghij", 4), vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn cjk_counts_double_width() {
        // Each CJK char is 2 columns; 4 columns fit 2 chars.
        assert_eq!(wrap_text("神愛世人", 4), vec!["神愛", "世人"]);
    }

    #[test]
    fn empty_text_yields_one_empty_line() {
        assert_eq!(wrap_text("", 10), vec![""]);
        assert_eq!(wrapped_height("", 10), 1);
    }

    #[test]
    fn zero_width_does_not_loop() {
        assert_eq!(wrap_text("abc", 0), vec!["abc"]);
    }
}

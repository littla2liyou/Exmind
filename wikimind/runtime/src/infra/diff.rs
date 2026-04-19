use similar::{ChangeTag, TextDiff};
use std::fmt::Write;

/// Generates a Unified Diff format string comparing old_text and new_text.
pub fn generate_diff(old_text: &str, new_text: &str) -> String {
    let diff = TextDiff::from_lines(old_text, new_text);
    let mut output = String::new();

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            let _ = write!(&mut output, "{}{}", sign, change);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::generate_diff;

    #[test]
    fn generates_insert_diff() {
        let old_text = "";
        let new_text = "added\n";

        assert_eq!(generate_diff(old_text, new_text), "+added\n");
    }

    #[test]
    fn generates_delete_diff() {
        let old_text = "removed\n";
        let new_text = "";

        assert_eq!(generate_diff(old_text, new_text), "-removed\n");
    }

    #[test]
    fn generates_replace_diff() {
        let old_text = "before\n";
        let new_text = "after\n";

        assert_eq!(generate_diff(old_text, new_text), "-before\n+after\n");
    }

    #[test]
    fn preserves_newline_and_equal_line_formatting() {
        let old_text = "same\nold";
        let new_text = "same\nnew";

        assert_eq!(generate_diff(old_text, new_text), " same\n-old+new");
    }
}

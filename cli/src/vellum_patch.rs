//! Content-minimal, UTF-8-safe single-replacement planning for Vellum edits.

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MinimalPatch {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) inserted: String,
}

impl MinimalPatch {
    pub(crate) fn changed(&self) -> bool {
        self.start != self.end || !self.inserted.is_empty()
    }
}

pub(crate) fn minimal_patch(current: &str, desired: &str) -> MinimalPatch {
    let prefix = current
        .chars()
        .zip(desired.chars())
        .take_while(|(left, right)| left == right)
        .map(|(value, _)| value.len_utf8())
        .sum::<usize>();

    let current_tail = &current[prefix..];
    let desired_tail = &desired[prefix..];
    let suffix = current_tail
        .chars()
        .rev()
        .zip(desired_tail.chars().rev())
        .take_while(|(left, right)| left == right)
        .map(|(value, _)| value.len_utf8())
        .sum::<usize>();

    MinimalPatch {
        start: prefix,
        end: current.len() - suffix,
        inserted: desired[prefix..desired.len() - suffix].to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(current: &str, patch: &MinimalPatch) -> String {
        format!(
            "{}{}{}",
            &current[..patch.start],
            patch.inserted,
            &current[patch.end..]
        )
    }

    #[test]
    fn unchanged_document_has_empty_patch() {
        let patch = minimal_patch("same", "same");
        assert_eq!(
            patch,
            MinimalPatch {
                start: 4,
                end: 4,
                inserted: String::new()
            }
        );
        assert!(!patch.changed());
    }

    #[test]
    fn replacement_preserves_common_prefix_and_suffix() {
        let patch = minimal_patch("alpha old omega", "alpha new omega");
        assert_eq!(patch.start, 6);
        assert_eq!(patch.end, 9);
        assert_eq!(patch.inserted, "new");
        assert_eq!(apply("alpha old omega", &patch), "alpha new omega");
    }

    #[test]
    fn unicode_boundaries_remain_valid() {
        let current = "a🙂oldλz";
        let desired = "a🙂newλz";
        let patch = minimal_patch(current, desired);
        assert!(current.is_char_boundary(patch.start));
        assert!(current.is_char_boundary(patch.end));
        assert_eq!(apply(current, &patch), desired);
    }

    #[test]
    fn insertion_and_deletion_are_minimal() {
        let insertion = minimal_patch("ab", "aXYb");
        assert_eq!(
            insertion,
            MinimalPatch {
                start: 1,
                end: 1,
                inserted: "XY".into()
            }
        );
        let deletion = minimal_patch("aXYb", "ab");
        assert_eq!(
            deletion,
            MinimalPatch {
                start: 1,
                end: 3,
                inserted: String::new()
            }
        );
    }
}

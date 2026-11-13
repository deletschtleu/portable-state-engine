//! Portable Event-Driven State Engine (`PSE-001`).

pub mod core;

/// Returns the specification identifier implemented by this crate.
pub fn spec_id() -> &'static str {
    "PSE-001"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_canonical_specification_id() {
        assert_eq!(spec_id(), "PSE-001");
    }
}

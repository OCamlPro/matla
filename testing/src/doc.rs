//! Defines and handles doc-tests.

prelude!();

/// Test kind for doc tests.
#[readonly]
pub struct Kind {
    /// Test source.
    pub source: source::LineSpan,
    /// Name of the entity the test is attached to.
    pub entity: String,
}
impl Kind {
    /// Constructor.
    pub fn new(entity: impl Into<String>, source: source::LineSpan) -> Self {
        Self {
            source,
            entity: entity.into(),
        }
    }
}

// impl DocTest {}

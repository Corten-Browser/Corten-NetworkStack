//! CT policy configuration

/// Certificate Transparency policy configuration
///
/// Defines the requirements for Certificate Transparency validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CtPolicy {
    /// Whether to require at least one SCT
    pub require_sct: bool,

    /// Minimum number of SCTs required
    pub min_sct_count: usize,
}

impl Default for CtPolicy {
    /// Create a new CT policy with default settings
    ///
    /// Default policy requires at least one SCT.
    fn default() -> Self {
        Self {
            require_sct: true,
            min_sct_count: 1,
        }
    }
}

impl CtPolicy {

    /// Create a lenient policy that doesn't require SCTs
    pub fn lenient() -> Self {
        Self {
            require_sct: false,
            min_sct_count: 0,
        }
    }

    /// Create a strict policy requiring multiple SCTs
    pub fn strict(min_count: usize) -> Self {
        Self {
            require_sct: true,
            min_sct_count: min_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_default() {
        let policy = CtPolicy::default();
        assert!(policy.require_sct);
        assert_eq!(policy.min_sct_count, 1);
    }

    #[test]
    fn test_policy_lenient() {
        let policy = CtPolicy::lenient();
        assert!(!policy.require_sct);
        assert_eq!(policy.min_sct_count, 0);
    }

    #[test]
    fn test_policy_strict() {
        let policy = CtPolicy::strict(3);
        assert!(policy.require_sct);
        assert_eq!(policy.min_sct_count, 3);
    }
}

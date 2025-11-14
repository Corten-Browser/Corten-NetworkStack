use network_types::ReferrerPolicy;

#[test]
fn test_referrer_policy_variants_exist() {
    // Given the ReferrerPolicy enum
    // When all referrer policy variants are defined
    // Then each variant should be accessible
    let _ = ReferrerPolicy::NoReferrer;
    let _ = ReferrerPolicy::NoReferrerWhenDowngrade;
    let _ = ReferrerPolicy::Origin;
    let _ = ReferrerPolicy::OriginWhenCrossOrigin;
    let _ = ReferrerPolicy::SameOrigin;
    let _ = ReferrerPolicy::StrictOrigin;
    let _ = ReferrerPolicy::StrictOriginWhenCrossOrigin;
    let _ = ReferrerPolicy::UnsafeUrl;
}

#[test]
fn test_referrer_policy_debug() {
    // Given a referrer policy
    // When debug formatted
    // Then it should produce readable output
    let policy = ReferrerPolicy::NoReferrer;
    let debug_str = format!("{:?}", policy);
    assert!(debug_str.contains("NoReferrer"));
}

#[test]
fn test_referrer_policy_clone() {
    // Given a referrer policy
    // When cloned
    // Then the clone should equal the original
    let policy = ReferrerPolicy::StrictOrigin;
    let cloned = policy;
    assert_eq!(policy, cloned);
}

#[test]
fn test_referrer_policy_partial_eq() {
    // Given two referrer policies
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(ReferrerPolicy::NoReferrer, ReferrerPolicy::NoReferrer);
    assert_eq!(ReferrerPolicy::Origin, ReferrerPolicy::Origin);
    assert_ne!(ReferrerPolicy::NoReferrer, ReferrerPolicy::Origin);
    assert_ne!(ReferrerPolicy::SameOrigin, ReferrerPolicy::StrictOrigin);
}

#[test]
fn test_referrer_policy_all_variants() {
    // Given all ReferrerPolicy variants
    // When verifying distinctness
    // Then each should be unique
    let policies = [
        ReferrerPolicy::NoReferrer,
        ReferrerPolicy::NoReferrerWhenDowngrade,
        ReferrerPolicy::Origin,
        ReferrerPolicy::OriginWhenCrossOrigin,
        ReferrerPolicy::SameOrigin,
        ReferrerPolicy::StrictOrigin,
        ReferrerPolicy::StrictOriginWhenCrossOrigin,
        ReferrerPolicy::UnsafeUrl,
    ];
    assert_eq!(policies.len(), 8);

    for (i, policy1) in policies.iter().enumerate() {
        for (j, policy2) in policies.iter().enumerate() {
            if i == j {
                assert_eq!(policy1, policy2);
            } else {
                assert_ne!(policy1, policy2);
            }
        }
    }
}

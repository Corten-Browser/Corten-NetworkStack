use network_types::CacheMode;

#[test]
fn test_cache_mode_variants_exist() {
    // Given the CacheMode enum
    // When all cache control variants are defined
    // Then each variant should be accessible
    let _ = CacheMode::Default;
    let _ = CacheMode::NoStore;
    let _ = CacheMode::Reload;
    let _ = CacheMode::NoCache;
    let _ = CacheMode::ForceCache;
    let _ = CacheMode::OnlyIfCached;
}

#[test]
fn test_cache_mode_debug() {
    // Given a cache mode
    // When debug formatted
    // Then it should produce readable output
    let mode = CacheMode::NoStore;
    let debug_str = format!("{:?}", mode);
    assert!(debug_str.contains("NoStore"));
}

#[test]
fn test_cache_mode_clone() {
    // Given a cache mode
    // When cloned
    // Then the clone should equal the original
    let mode = CacheMode::ForceCache;
    let cloned = mode;
    assert_eq!(mode, cloned);
}

#[test]
fn test_cache_mode_partial_eq() {
    // Given two cache modes
    // When compared for equality
    // Then same variants should be equal
    // And different variants should not be equal
    assert_eq!(CacheMode::Default, CacheMode::Default);
    assert_eq!(CacheMode::NoStore, CacheMode::NoStore);
    assert_ne!(CacheMode::Default, CacheMode::NoStore);
    assert_ne!(CacheMode::Reload, CacheMode::NoCache);
}

#[test]
fn test_cache_mode_all_variants() {
    // Given all CacheMode variants
    // When verifying distinctness
    // Then each should be unique
    let modes = [
        CacheMode::Default,
        CacheMode::NoStore,
        CacheMode::Reload,
        CacheMode::NoCache,
        CacheMode::ForceCache,
        CacheMode::OnlyIfCached,
    ];
    assert_eq!(modes.len(), 6);

    for (i, mode1) in modes.iter().enumerate() {
        for (j, mode2) in modes.iter().enumerate() {
            if i == j {
                assert_eq!(mode1, mode2);
            } else {
                assert_ne!(mode1, mode2);
            }
        }
    }
}

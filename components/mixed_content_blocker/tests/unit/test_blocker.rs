use mixed_content_blocker::{
    ContentType, MixedContentBlocker, MixedContentPolicy,
};
use url::Url;

#[test]
fn test_https_page_blocks_http_active_content() {
    // Given: HTTPS page and policy to block mixed content
    let policy = MixedContentPolicy {
        block_all_mixed_content: true,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("https://example.com/page").unwrap();
    let resource_url = Url::parse("http://example.com/script.js").unwrap();

    // When: Checking HTTP script in HTTPS page
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Active);

    // Then: Should be blocked
    assert!(result.blocked);
    assert!(result.reason.is_some());
    assert!(result.reason.unwrap().contains("Mixed content"));
}

#[test]
fn test_https_page_allows_http_passive_content_when_not_strict() {
    // Given: HTTPS page with permissive policy
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("https://example.com/page").unwrap();
    let resource_url = Url::parse("http://example.com/image.png").unwrap();

    // When: Checking HTTP image in HTTPS page
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Passive);

    // Then: Should not be blocked (warning only)
    assert!(!result.blocked);
    assert!(result.reason.is_some()); // Still has warning
}

#[test]
fn test_https_page_blocks_http_passive_content_when_strict() {
    // Given: HTTPS page with strict policy
    let policy = MixedContentPolicy {
        block_all_mixed_content: true,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("https://example.com/page").unwrap();
    let resource_url = Url::parse("http://example.com/image.png").unwrap();

    // When: Checking HTTP image with strict policy
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Passive);

    // Then: Should be blocked
    assert!(result.blocked);
}

#[test]
fn test_http_page_allows_http_resources() {
    // Given: HTTP page
    let policy = MixedContentPolicy {
        block_all_mixed_content: true,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("http://example.com/page").unwrap();
    let resource_url = Url::parse("http://example.com/script.js").unwrap();

    // When: Checking HTTP resource in HTTP page
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Active);

    // Then: Should not be blocked
    assert!(!result.blocked);
    assert!(result.reason.is_none());
}

#[test]
fn test_https_page_allows_https_resources() {
    // Given: HTTPS page
    let policy = MixedContentPolicy {
        block_all_mixed_content: true,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("https://example.com/page").unwrap();
    let resource_url = Url::parse("https://example.com/script.js").unwrap();

    // When: Checking HTTPS resource in HTTPS page
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Active);

    // Then: Should not be blocked
    assert!(!result.blocked);
    assert!(result.reason.is_none());
}

#[test]
fn test_upgrade_insecure_requests_upgrades_http_to_https() {
    // Given: Policy with upgrade-insecure-requests enabled
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let page_url = Url::parse("https://example.com/page").unwrap();
    let resource_url = Url::parse("http://example.com/script.js").unwrap();

    // When: Checking HTTP resource with upgrade enabled
    let result = blocker.check_request(&page_url, &resource_url, ContentType::Active);

    // Then: Should not be blocked and should have upgraded URL
    assert!(!result.blocked);
    assert!(result.upgraded_url.is_some());
    let upgraded = result.upgraded_url.unwrap();
    assert_eq!(upgraded.scheme(), "https");
    assert_eq!(upgraded.path(), "/script.js");
}

#[test]
fn test_should_upgrade_returns_true_for_http_with_policy() {
    // Given: Policy with upgrade enabled and HTTP URL
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let url = Url::parse("http://example.com/resource").unwrap();

    // When: Checking if should upgrade
    let should_upgrade = blocker.should_upgrade(&url);

    // Then: Should return true
    assert!(should_upgrade);
}

#[test]
fn test_should_upgrade_returns_false_for_https() {
    // Given: Policy with upgrade enabled but HTTPS URL
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let url = Url::parse("https://example.com/resource").unwrap();

    // When: Checking if should upgrade
    let should_upgrade = blocker.should_upgrade(&url);

    // Then: Should return false (already secure)
    assert!(!should_upgrade);
}

#[test]
fn test_should_upgrade_returns_false_when_policy_disabled() {
    // Given: Policy with upgrade disabled
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);
    let url = Url::parse("http://example.com/resource").unwrap();

    // When: Checking if should upgrade
    let should_upgrade = blocker.should_upgrade(&url);

    // Then: Should return false (policy disabled)
    assert!(!should_upgrade);
}

#[test]
fn test_upgrade_to_https_converts_http_to_https() {
    // Given: Blocker with any policy
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let http_url = Url::parse("http://example.com:8080/path?query=value#fragment").unwrap();

    // When: Upgrading to HTTPS
    let result = blocker.upgrade_to_https(&http_url);

    // Then: Should successfully upgrade
    assert!(result.is_ok());
    let https_url = result.unwrap();
    assert_eq!(https_url.scheme(), "https");
    assert_eq!(https_url.host_str(), Some("example.com"));
    assert_eq!(https_url.port(), Some(8080));
    assert_eq!(https_url.path(), "/path");
    assert_eq!(https_url.query(), Some("query=value"));
    assert_eq!(https_url.fragment(), Some("fragment"));
}

#[test]
fn test_upgrade_to_https_fails_for_non_http_schemes() {
    // Given: Blocker and non-HTTP URL
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let ftp_url = Url::parse("ftp://example.com/file").unwrap();

    // When: Attempting to upgrade non-HTTP URL
    let result = blocker.upgrade_to_https(&ftp_url);

    // Then: Should fail
    assert!(result.is_err());
}

#[test]
fn test_upgrade_to_https_is_noop_for_https() {
    // Given: Already HTTPS URL
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let blocker = MixedContentBlocker::new(policy);
    let https_url = Url::parse("https://example.com/path").unwrap();

    // When: Upgrading HTTPS URL
    let result = blocker.upgrade_to_https(&https_url);

    // Then: Should return same URL
    assert!(result.is_ok());
    let upgraded = result.unwrap();
    assert_eq!(upgraded.as_str(), https_url.as_str());
}

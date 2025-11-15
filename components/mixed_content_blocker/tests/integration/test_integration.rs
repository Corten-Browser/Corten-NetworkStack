use mixed_content_blocker::{
    ContentType, MixedContentBlocker, MixedContentPolicy,
};
use url::Url;

#[test]
fn test_end_to_end_mixed_content_blocking() {
    // Integration test: Full workflow of mixed content detection and blocking

    // Scenario 1: Strict policy blocks all mixed content
    let strict_policy = MixedContentPolicy {
        block_all_mixed_content: true,
        upgrade_insecure_requests: false,
    };
    let strict_blocker = MixedContentBlocker::new(strict_policy);

    let https_page = Url::parse("https://secure.example.com").unwrap();
    let http_script = Url::parse("http://insecure.example.com/script.js").unwrap();
    let http_image = Url::parse("http://insecure.example.com/image.png").unwrap();

    // Active content should be blocked
    let script_result = strict_blocker.check_request(&https_page, &http_script, ContentType::Active);
    assert!(script_result.blocked);

    // Passive content should also be blocked with strict policy
    let image_result = strict_blocker.check_request(&https_page, &http_image, ContentType::Passive);
    assert!(image_result.blocked);

    // Scenario 2: Upgrade policy upgrades instead of blocking
    let upgrade_policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: true,
    };
    let upgrade_blocker = MixedContentBlocker::new(upgrade_policy);

    let script_result = upgrade_blocker.check_request(&https_page, &http_script, ContentType::Active);
    assert!(!script_result.blocked);
    assert!(script_result.upgraded_url.is_some());
    assert_eq!(script_result.upgraded_url.unwrap().scheme(), "https");

    // Scenario 3: Permissive policy warns but doesn't block passive content
    let permissive_policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: false,
    };
    let permissive_blocker = MixedContentBlocker::new(permissive_policy);

    // Active content still blocked
    let script_result = permissive_blocker.check_request(&https_page, &http_script, ContentType::Active);
    assert!(script_result.blocked);

    // Passive content warned but not blocked
    let image_result = permissive_blocker.check_request(&https_page, &http_image, ContentType::Passive);
    assert!(!image_result.blocked);
    assert!(image_result.reason.is_some());
}

#[test]
fn test_various_content_types_classification() {
    // Test that different content types are handled correctly
    let policy = MixedContentPolicy {
        block_all_mixed_content: false,
        upgrade_insecure_requests: false,
    };
    let blocker = MixedContentBlocker::new(policy);

    let https_page = Url::parse("https://example.com").unwrap();
    let http_resource = Url::parse("http://example.com/resource").unwrap();

    // Active content (scripts, stylesheets, objects) should be blocked
    let active_result = blocker.check_request(&https_page, &http_resource, ContentType::Active);
    assert!(active_result.blocked);

    // Passive content (images, audio, video) should only warn
    let passive_result = blocker.check_request(&https_page, &http_resource, ContentType::Passive);
    assert!(!passive_result.blocked);
    assert!(passive_result.reason.is_some());
}

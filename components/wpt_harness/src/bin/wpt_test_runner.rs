//! Unified WPT Test Runner for Phase 3
//!
//! Runs all test suites (HTTP, CORS, CSP) and generates combined report

use wpt_harness::{http_tests, cors_tests, csp_tests};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   Corten-NetworkStack WPT Integration Phase 3               ║");
    println!("║   Comprehensive Multi-Protocol Test Suite                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Test Target: http://127.0.0.1:8080 (local test server)");
    println!("Protocol: HTTP/1.1 via NetworkStack");
    println!("Test Categories: HTTP, CORS, CSP\n");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Run HTTP test suite
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   HTTP PROTOCOL TESTS                                     ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    let http_stats = http_tests::run_http_test_suite().await;

    println!("\n═══════════════════════════════════════════════════════════════\n");

    // Run CORS test suite
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   CORS (Cross-Origin Resource Sharing) TESTS             ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    let cors_stats = cors_tests::run_cors_test_suite().await;

    println!("\n═══════════════════════════════════════════════════════════════\n");

    // Run CSP test suite
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║   CSP (Content Security Policy) TESTS                    ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");
    let csp_stats = csp_tests::run_csp_test_suite().await;

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 COMBINED RESULTS - PHASE 3                   ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Calculate combined statistics
    let total_tests = http_stats.total + cors_stats.total + csp_stats.total;
    let total_passed = http_stats.passed + cors_stats.passed + csp_stats.passed;
    let total_failed = http_stats.failed + cors_stats.failed + csp_stats.failed;
    let total_errors = http_stats.errors + cors_stats.errors + csp_stats.errors;
    let combined_pass_rate = if total_tests > 0 {
        (total_passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    println!("Category Breakdown:");
    println!("  HTTP:    {}/{} passed ({:.1}%)", http_stats.passed, http_stats.total, http_stats.pass_rate());
    println!("  CORS:    {}/{} passed ({:.1}%)", cors_stats.passed, cors_stats.total, cors_stats.pass_rate());
    println!("  CSP:     {}/{} passed ({:.1}%)", csp_stats.passed, csp_stats.total, csp_stats.pass_rate());
    println!();

    println!("Overall Results:");
    println!("  Total Tests:     {}", total_tests);
    println!("  Passed:          {} ({:.1}%)", total_passed, combined_pass_rate);
    println!("  Failed:          {}", total_failed);
    println!("  Errors:          {}", total_errors);
    println!();

    // Determine if we met Phase 3 target
    let target_pass_rate = 90.0;
    if combined_pass_rate >= target_pass_rate {
        println!("✅ SUCCESS: Met Phase 3 target of {:.0}% pass rate!", target_pass_rate);
        println!("   Multi-protocol NetworkStack validation complete.");
    } else {
        println!("⚠️  BELOW TARGET: Pass rate {:.1}% is below target {:.0}%", combined_pass_rate, target_pass_rate);
        println!("   Review failures and improve implementation.");
    }

    println!("\n═══════════════════════════════════════════════════════════════");

    // Generate detailed JSON report
    let report = serde_json::json!({
        "version": "0.3.0",
        "phase": 3,
        "test_suite": "multi_protocol_integration",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "categories": {
            "http": {
                "total": http_stats.total,
                "passed": http_stats.passed,
                "failed": http_stats.failed,
                "errors": http_stats.errors,
                "pass_rate": http_stats.pass_rate(),
            },
            "cors": {
                "total": cors_stats.total,
                "passed": cors_stats.passed,
                "failed": cors_stats.failed,
                "errors": cors_stats.errors,
                "pass_rate": cors_stats.pass_rate(),
            },
            "csp": {
                "total": csp_stats.total,
                "passed": csp_stats.passed,
                "failed": csp_stats.failed,
                "errors": csp_stats.errors,
                "pass_rate": csp_stats.pass_rate(),
            },
        },
        "combined": {
            "total": total_tests,
            "passed": total_passed,
            "failed": total_failed,
            "errors": total_errors,
            "pass_rate": combined_pass_rate,
        },
        "target_pass_rate": target_pass_rate,
        "target_met": combined_pass_rate >= target_pass_rate,
    });

    println!("\nJSON Report:");
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}

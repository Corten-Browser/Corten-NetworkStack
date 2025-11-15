//! HTTP Test Runner for WPT Integration v0.2.0
//!
//! Runs a comprehensive HTTP test suite that validates NetworkStack functionality
//! equivalent to WPT fetch/xhr tests, without requiring a browser environment.

use wpt_harness::http_tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║   Corten-NetworkStack WPT Integration v0.2.0                ║");
    println!("║   HTTP Test Suite - NetworkStack API Bridge                 ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    println!("Test Target: http://127.0.0.1:8080 (local test server)");
    println!("Protocol: HTTP/1.1 via NetworkStack");
    println!("Test Categories: fetch, xhr, status codes, headers, encoding\n");
    println!("═══════════════════════════════════════════════════════════════\n");

    // Run the HTTP test suite
    let stats = http_tests::run_http_test_suite().await;

    println!("\n═══════════════════════════════════════════════════════════════");
    stats.print_summary();

    // Determine if we met v0.2.0 target
    let target_pass_rate = 85.0;
    if stats.pass_rate() >= target_pass_rate {
        println!("\n✅ SUCCESS: Met v0.2.0 target of {}% pass rate!", target_pass_rate);
        println!("   NetworkStack API bridge is functional and validated.");
    } else {
        println!("\n⚠️  BELOW TARGET: Pass rate {:.1}% is below target {}%", stats.pass_rate(), target_pass_rate);
        println!("   Review failures and improve implementation.");
    }

    println!("\n═══════════════════════════════════════════════════════════════");

    // Generate simple JSON report
    let report = serde_json::json!({
        "version": "0.2.0",
        "test_suite": "http_integration",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "total": stats.total,
        "passed": stats.passed,
        "failed": stats.failed,
        "errors": stats.errors,
        "pass_rate": stats.pass_rate(),
        "target_pass_rate": target_pass_rate,
        "target_met": stats.pass_rate() >= target_pass_rate,
    });

    println!("\nJSON Report:");
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}

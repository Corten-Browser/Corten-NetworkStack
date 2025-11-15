# WPT Harness

Web Platform Tests (WPT) integration adapter for Corten-NetworkStack.

## Overview

This component provides integration between the Corten-NetworkStack and the Web Platform Tests suite, enabling validation of network stack functionality against web standards.

## Architecture

```
WPT Test Server (Python)
     ↓ HTTP
WPT Harness Adapter (this crate)
     ↓ Rust API
NetworkStack (core implementation)
```

## Status

**Version**: 0.1.0 (Proof-of-Concept)
**WPT Repository**: Cloned (99,806 files, 266MB at `/home/user/wpt/`)
**Test Categories**: 6 categories, 2,108 test files identified

### Current Implementation

✅ **Infrastructure**:
- WPT repository cloned and analyzed
- Test request/response data structures
- Test result types and statistics tracking
- Sample runner binary

⚠️ **Pending**:
- Full NetworkStack API integration
- WPT protocol adapter implementation
- Automated test execution

## Usage

### Build the Runner

```bash
cargo build --release --bin wpt_runner
```

### Run Sample Tests

```bash
# Run with default output
./target/release/wpt_runner

# Run with verbose output
./target/release/wpt_runner --verbose
```

### Current Output (Proof-of-Concept)

```
Corten-NetworkStack WPT Test Runner
====================================

Running 3 sample tests...

  basic_get ... PASS
  with_headers ... PASS
  post_request ... PASS

WPT Test Results:
  Total:    3
  Passed:   3 (100%)
  Failed:   0
  Timeout:  0
  Skipped:  0
  Errors:   0
```

## Integration Plan

See **`docs/WPT-INTEGRATION-PLAN.md`** for comprehensive integration plan:

- **Phase 1** ✅: Documentation & Planning (Complete)
- **Phase 2** 🔄: Sample Implementation (In Progress)
- **Phase 3** ⏳: Full Execution (Planned)

## Test Categories

| Category | Tests | Target | Priority |
|----------|-------|--------|----------|
| fetch | 591 | 90% | Essential |
| xhr | 154 | 90% | Essential |
| websockets | 222 | 95% | Essential |
| cors | 10 | 95% | Essential |
| mixed-content | 167 | 100% | Security |
| CSP | 964 | 95% | Security |

**Total**: 2,108 test files

## API

### WptHarness

Main harness adapter:

```rust
use wpt_harness::{WptHarness, WptRequest};

let harness = WptHarness::new();
let request = WptRequest {
    method: "GET".to_string(),
    url: "https://example.com".to_string(),
    headers: Default::default(),
    body: None,
    timeout_ms: Some(30000),
};

let response = harness.execute_request(request).await?;
```

### WptTestStats

Test statistics tracker:

```rust
use wpt_harness::{WptTestStats, WptTestResult};

let mut stats = WptTestStats::default();
stats.add_result(&WptTestResult::Pass);
stats.print_summary();
```

## Development

### Run Tests

```bash
cargo test -p wpt_harness
```

### Current Test Coverage

- ✅ 6 unit tests
- ✅ 100% pass rate
- ✅ Harness creation, configuration, request execution
- ✅ Statistics tracking and calculation

## Next Steps

### For v0.2.0

1. Implement NetworkStack API integration
2. Create WPT protocol adapter
3. Run automated tests for core categories (fetch, xhr, websockets)
4. Achieve 85%+ pass rate

### For v1.0.0

1. Complete integration for all categories
2. Execute full WPT suite (2,108 tests)
3. Achieve 90%+ overall pass rate
4. Generate WPT compliance report

## References

- **WPT Repository**: https://github.com/web-platform-tests/wpt
- **Integration Plan**: `docs/WPT-INTEGRATION-PLAN.md`
- **Network Stack Spec**: `network-stack-specification.md`

## License

See project root LICENSE file.

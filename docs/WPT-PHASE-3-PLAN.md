# WPT Integration Phase 3 - Implementation Plan
**Project**: Corten-NetworkStack
**Version**: 0.3.0
**Date**: 2025-11-15
**Status**: Ready to Execute

---

## Executive Summary

Phase 3 expands WPT integration from 21 HTTP tests to **100+ comprehensive tests** covering WebSocket, CORS, and CSP protocols.

**Current Status**:
- ✅ Phase 1: Documentation & Planning - COMPLETE
- ✅ Phase 2: HTTP Testing (21 tests, 100% pass rate) - COMPLETE
- 🚀 Phase 3: Multi-Protocol Testing (100+ tests) - **IN PROGRESS**

**Goals**:
- Add WebSocket test suite (20-30 tests)
- Add CORS test suite (10-15 tests)
- Add CSP test suite (15-20 tests)
- Achieve 90%+ pass rate across all categories
- Validate multi-protocol NetworkStack functionality

---

## Test Suite Expansion Plan

### Category 1: WebSocket Tests (20-30 tests)

**Priority**: High (WebSocket is essential protocol)

**Test Coverage**:

1. **Connection Tests** (5 tests)
   - Basic connection establishment (ws://)
   - Secure connection (wss://)
   - Connection with subprotocols
   - Connection failure (invalid URL)
   - Connection timeout

2. **Message Tests** (8 tests)
   - Send text message
   - Receive text message
   - Send binary message
   - Receive binary message
   - Send empty message
   - Send large message (>64KB)
   - Message fragmentation
   - Ping/pong frames

3. **Close Handshake Tests** (4 tests)
   - Normal close (code 1000)
   - Client-initiated close
   - Server-initiated close
   - Close with reason

4. **Error Handling Tests** (5 tests)
   - Invalid frame
   - Protocol error
   - Connection drop
   - Max message size exceeded
   - Invalid UTF-8 in text frame

5. **Advanced Features** (3 tests)
   - Compression (permessage-deflate)
   - Multiple connections
   - Connection reuse

**Total**: 25 tests

### Category 2: CORS Tests (10-15 tests)

**Priority**: High (Security critical)

**Test Coverage**:

1. **Simple Requests** (3 tests)
   - Same-origin request (allowed)
   - Cross-origin GET (with Origin header)
   - Cross-origin POST (simple content-type)

2. **Preflight Requests** (4 tests)
   - OPTIONS preflight
   - Custom headers (X-Custom-Header)
   - Non-simple methods (PUT, DELETE)
   - Preflight with credentials

3. **Response Headers** (3 tests)
   - Access-Control-Allow-Origin validation
   - Access-Control-Allow-Methods validation
   - Access-Control-Allow-Headers validation

4. **Credentials Mode** (3 tests)
   - Credentials: omit
   - Credentials: same-origin
   - Credentials: include

5. **Error Cases** (2 tests)
   - Missing CORS headers (blocked)
   - Wildcard with credentials (blocked)

**Total**: 15 tests

### Category 3: CSP Tests (15-20 tests)

**Priority**: High (Security critical)

**Test Coverage**:

1. **Directive Parsing** (4 tests)
   - default-src directive
   - script-src directive
   - style-src directive
   - img-src directive

2. **Source Lists** (4 tests)
   - 'self' keyword
   - 'none' keyword
   - Specific origins (https://example.com)
   - Wildcard sources (*.example.com)

3. **Nonce Validation** (3 tests)
   - Valid nonce match
   - Invalid nonce (blocked)
   - Multiple nonces

4. **Hash Validation** (3 tests)
   - SHA-256 hash match
   - SHA-384 hash match
   - Invalid hash (blocked)

5. **Violation Reporting** (3 tests)
   - report-uri directive
   - report-to directive
   - Violation report format

6. **Advanced Features** (3 tests)
   - 'unsafe-inline' keyword
   - 'unsafe-eval' keyword
   - Multiple policies

**Total**: 20 tests

---

## Implementation Strategy

### Phase 3.1: Infrastructure Setup

**Tasks**:
1. Extend test_server.py to support WebSocket
2. Add CORS header support to test server
3. Add CSP header support to test server
4. Create test endpoint structure

**Estimated Time**: 2-3 hours

### Phase 3.2: WebSocket Tests

**Tasks**:
1. Create `components/wpt_harness/src/websocket_tests.rs`
2. Implement 25 WebSocket test cases
3. Integrate WebSocket protocol component
4. Test WebSocket connection lifecycle

**Estimated Time**: 3-4 hours

### Phase 3.3: CORS Tests

**Tasks**:
1. Create `components/wpt_harness/src/cors_tests.rs`
2. Implement 15 CORS test cases
3. Integrate CORS validator component
4. Test preflight and simple requests

**Estimated Time**: 2-3 hours

### Phase 3.4: CSP Tests

**Tasks**:
1. Create `components/wpt_harness/src/csp_tests.rs`
2. Implement 20 CSP test cases
3. Integrate CSP processor component
4. Test directive parsing and enforcement

**Estimated Time**: 2-3 hours

### Phase 3.5: Integration and Testing

**Tasks**:
1. Create unified test runner
2. Run complete test suite (60+ tests)
3. Analyze failures
4. Fix NetworkStack issues
5. Re-test until 90%+ pass rate

**Estimated Time**: 2-3 hours

### Phase 3.6: Documentation

**Tasks**:
1. Generate Phase 3 compliance report
2. Update WPT integration plan
3. Document any spec deviations
4. Create deployment guide

**Estimated Time**: 1-2 hours

**Total Estimated Time**: 12-18 hours

---

## Test Server Architecture

### Extended Test Server (test_server.py)

**Current Features**:
- HTTP/1.1 endpoints (GET, POST, PUT, DELETE, PATCH)
- Status code simulation
- Header manipulation
- Content encoding (gzip, deflate)
- Delays and timeouts

**New Features Required**:

1. **WebSocket Support**
   - WebSocket upgrade handler
   - Text/binary message echo
   - Close handshake
   - Ping/pong
   - Protocol error simulation

2. **CORS Support**
   - Configurable CORS headers
   - Preflight response
   - Origin validation
   - Credentials handling

3. **CSP Support**
   - CSP header generation
   - Policy combination
   - Violation simulation
   - Report-URI endpoint

### Alternative: Separate Test Servers

**Option**: Run multiple specialized servers
- HTTP server: port 8080 (existing)
- WebSocket server: port 9000
- CORS test server: port 8081
- CSP test server: port 8082

**Recommendation**: Start with single server, split if needed

---

## NetworkStack Component Integration

### Components to Test

1. **websocket_protocol** (`components/websocket_protocol/`)
   - WebSocket client implementation
   - Connection management
   - Message framing
   - Close handshake

2. **cors_validator** (`components/cors_validator/`)
   - CORS policy validation
   - Preflight checking
   - Origin validation
   - Credentials mode enforcement

3. **csp_processor** (`components/csp_processor/`)
   - CSP header parsing
   - Directive evaluation
   - Nonce/hash validation
   - Violation reporting

### Integration Points

**WPT Harness Extensions**:
```rust
// components/wpt_harness/src/lib.rs

// Add WebSocket support
pub async fn execute_websocket_test(
    &self,
    test: WsTestCase,
) -> Result<WsTestResult, Box<dyn std::error::Error>>

// Add CORS validation
pub async fn validate_cors_request(
    &self,
    request: WptRequest,
) -> Result<CorsResult, Box<dyn std::error::Error>>

// Add CSP validation
pub fn validate_csp_policy(
    &self,
    policy: &str,
) -> Result<CspResult, Box<dyn std::error::Error>>
```

---

## Success Criteria

### Phase 3 Completion Criteria

**Minimum Requirements**:
- ✅ 60+ total tests created (HTTP + WebSocket + CORS + CSP)
- ✅ 90%+ overall pass rate
- ✅ All critical protocols tested (HTTP, WebSocket, CORS, CSP)
- ✅ Zero security vulnerabilities found
- ✅ Comprehensive documentation

**Stretch Goals**:
- 🎯 100+ total tests
- 🎯 95%+ pass rate
- 🎯 HTTP/2 basic tests (10 tests)
- 🎯 Performance benchmarks

### Quality Gates

**Before Completion**:
1. All tests execute successfully (no crashes)
2. Pass rate ≥ 90% across all categories
3. No regression in Phase 2 tests (21 HTTP tests still 100%)
4. Security audit passed
5. Documentation complete

---

## Risk Assessment

### Potential Challenges

1. **WebSocket Server Implementation**
   - Risk: Python WebSocket support complexity
   - Mitigation: Use `websockets` library (well-tested)
   - Fallback: Use Node.js ws library

2. **CORS Preflight Complexity**
   - Risk: Preflight logic may be complex
   - Mitigation: Test with browser dev tools first
   - Fallback: Simplify test scenarios

3. **CSP Policy Parsing**
   - Risk: CSP syntax is complex
   - Mitigation: Use existing CSP parser tests
   - Fallback: Test subset of directives

4. **Integration with Existing Components**
   - Risk: Components may have bugs
   - Mitigation: Thorough component testing
   - Fallback: Fix components as issues found

### Contingency Plans

**If pass rate < 90%**:
1. Identify failing test patterns
2. Fix critical failures first
3. Document known issues
4. Plan fixes for v0.3.1

**If implementation takes > 20 hours**:
1. Reduce scope (fewer tests per category)
2. Focus on critical tests only
3. Defer stretch goals to v0.4.0

---

## Deliverables

### Code Deliverables

1. **Extended test_server.py**
   - WebSocket support
   - CORS support
   - CSP support

2. **WebSocket Test Suite**
   - `components/wpt_harness/src/websocket_tests.rs` (25 tests)

3. **CORS Test Suite**
   - `components/wpt_harness/src/cors_tests.rs` (15 tests)

4. **CSP Test Suite**
   - `components/wpt_harness/src/csp_tests.rs` (20 tests)

5. **Unified Test Runner**
   - `components/wpt_harness/src/bin/wpt_test_runner.rs`
   - Runs all test suites
   - Generates combined report

### Documentation Deliverables

1. **Phase 3 Compliance Report**
   - `docs/WPT-PHASE-3-COMPLIANCE-REPORT.md`
   - Detailed test results
   - Standards compliance analysis
   - Known issues and limitations

2. **Updated Integration Plan**
   - `docs/WPT-INTEGRATION-PLAN.md`
   - Phase 3 marked complete
   - Phase 4 roadmap

3. **Test Execution Guide**
   - How to run tests locally
   - How to extend test suite
   - Troubleshooting guide

---

## Timeline

### Day 1: Infrastructure (2-4 hours)
- ✅ Create Phase 3 plan (this document)
- 🚀 Extend test_server.py with WebSocket support
- 🚀 Add CORS and CSP endpoints

### Day 2: WebSocket Tests (3-4 hours)
- Create websocket_tests.rs
- Implement 25 test cases
- Run and validate

### Day 3: CORS & CSP Tests (4-6 hours)
- Create cors_tests.rs (15 tests)
- Create csp_tests.rs (20 tests)
- Run and validate

### Day 4: Integration & Fixes (2-4 hours)
- Create unified runner
- Fix failures
- Achieve 90%+ pass rate

### Day 5: Documentation (1-2 hours)
- Generate compliance report
- Update documentation
- Commit and push

**Total**: 12-20 hours (distributed over 5 work sessions)

---

## Next Steps (Immediate)

1. ✅ Create this plan document
2. 🚀 Research Python WebSocket libraries
3. 🚀 Design WebSocket test server architecture
4. 🚀 Implement basic WebSocket echo server
5. 🚀 Create first WebSocket test

**Auto-proceed to implementation**: Starting with WebSocket test server setup...

---

**Document Version**: 1.0
**Last Updated**: 2025-11-15
**Status**: 🚀 **READY TO EXECUTE**

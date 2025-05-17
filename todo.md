# TODO

## Test Section

### Web Service Test Coverage
- [ ] Add unit tests for all API endpoints in `src/web/handlers.rs`
  - [ ] `POST /sign` - Test signing with valid/invalid inputs
  - [ ] `POST /scalar_mul` - Test scalar multiplication endpoint
  - [ ] `GET /key/:eoa_address` - Test individual key retrieval
  - [ ] `GET /keys` - Test listing all keys
  - [ ] Add test coverage for error responses (404, 400, 500)

### Error Handling Tests
- [ ] Test invalid EOA addresses
- [ ] Test malformed request bodies
- [ ] Test missing required fields in requests
- [ ] Test invalid hex string formats
- [ ] Test key store loading failures
- [ ] Test concurrent request handling

### Edge Case Tests
- [ ] Add tests for point at infinity handling
- [ ] Test invalid curve points
- [ ] Test large scalar values (near field modulus)
- [ ] Test boundary conditions for field elements

### Code Quality Improvements
- [ ] Fix compilation warnings
  - [ ] Remove unused import `header` in lib.rs
  - [ ] Remove or implement unused methods `to_g1_point` and `to_g2_point`
  - [ ] Fix unused variable `anvil` in tests
- [ ] Add documentation tests for public API methods
- [ ] Consider adding code coverage reporting

### Performance Testing
- [ ] Add benchmarks for cryptographic operations
  - [ ] Scalar multiplication performance
  - [ ] Pairing operation performance
  - [ ] Hash to G1 performance
- [ ] Add load tests for web service
  - [ ] Concurrent request handling
  - [ ] Memory usage under load
  - [ ] Response time consistency

### Property-Based Testing Enhancements
- [ ] Add property-based tests for web API
  - [ ] Generate random valid/invalid requests
  - [ ] Verify response format consistency
  - [ ] Check cryptographic invariants (points on curve, valid signatures)
- [ ] Extend existing property tests
  - [ ] More edge cases for scalar multiplication
  - [ ] Random G1/G2 point generation and validation

### Integration Test Improvements
- [ ] Test all API endpoints in integration tests, not just `/registration-params`
- [ ] Add negative test cases (expected failures)
- [ ] Test API with malformed/invalid data
- [ ] Test API rate limiting and timeouts
- [ ] Add tests for the full signing workflow

### Test Infrastructure
- [ ] Consider adding test fixtures for common test data
- [ ] Create test utilities for generating valid/invalid keys
- [ ] Add helper functions for API testing
- [ ] Consider using test containers for integration tests
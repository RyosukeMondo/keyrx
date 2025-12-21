# Test Scripts

Scripts that **execute tests** and verify functionality.

## Principle

Test scripts run automated tests and fail if tests don't pass.

## Scripts

| Script | Tests | Fails If |
|--------|-------|----------|
| `daemon.sh` | Daemon integration tests | Test failures |
| `e2e.sh` | End-to-end UI tests | Test failures |
| `contracts.sh` | API contract tests | Contract violations |

## Usage

```bash
# Run daemon tests
./scripts/test/daemon.sh

# Run end-to-end tests
./scripts/test/e2e.sh

# Run contract tests
./scripts/test/contracts.sh

# Via justfile
just test-daemon
just test-e2e
just test-contracts
```

## Exit Codes

- `0` - All tests passed ✅
- `1` - Test failures detected ❌

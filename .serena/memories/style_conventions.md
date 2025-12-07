# Style & Conventions
- Rust: rustfmt + clippy enforced; pre-commit hooks run cargo fmt --check, cargo clippy -- -D warnings, cargo test --lib. Prefer idiomatic async with Tokio; avoid global state per architecture doc.
- Testing philosophy: layered quality (unit, property fuzzing, deterministic replay, integration via mock input, latency benchmarks). Requirements traceability via test metadata tags (category/priority/requirement/latency) in Rhai UAT tests.
- Flutter/Dart: follow standard Flutter patterns; use hot reload for iteration (just ui). Keep UI in ui/ tree.
- Spec workflow: update tasks.md statuses ([ ], [-], [x]); log implementations via log-implementation tool with artifacts.

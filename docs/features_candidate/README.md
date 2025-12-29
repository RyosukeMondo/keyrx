# Feature Candidates

This directory contains documentation for features that were **considered but deferred** for the KeyRx project. Each document provides:

- Technical specification of the proposed feature
- Pros and cons analysis
- Benchmark data (if applicable)
- Recommendation (implement, defer, or reject)
- Conditions for revisiting the decision

## Purpose

These documents serve as:

1. **Decision Records**: Why certain features were not implemented
2. **Future Reference**: Context for revisiting decisions later
3. **Knowledge Base**: Technical analysis that may inform future work
4. **Avoiding Rework**: Prevent re-analyzing the same features repeatedly

## Current Feature Candidates

### [MPHF Lookup System](./mphf-lookup-system.md)
**Status:** Deferred
**Date:** 2025-12-29
**Reason:** Current HashMap performance (4.7ns) already exceeds requirements by 21,000x. MPHF would provide only 2ns improvement for ~1000 lines of complex code. The real bottleneck is OS I/O (1-10μs), not lookup.

**Revisit if:**
- Profiling shows lookup is >10% of total latency
- Targeting embedded systems with <1MB RAM
- Safety-critical formal verification is required

### [CheckBytes Security Validation](./checkbytes-security.md)
**Status:** Deferred (Medium Priority Security Hardening)
**Date:** 2025-12-29
**Reason:** Current SHA256 hash validation + panic recovery already provides robust protection against corrupted .krx files. CheckBytes would improve error messages and provide defense-in-depth, but requires 1-2 days effort for minimal additional security benefit in current use case (user-generated configs only).

**Revisit if:**
- User-reported issues with corrupted .krx files
- Implementing config sharing or untrusted .krx loading
- Security audit requires defense-in-depth validation
- Better error messages needed for debugging

---

## Document Template

When adding new feature candidates, use this structure:

```markdown
# Feature Candidate: [Feature Name]

**Status:** [Deferred | Rejected | Approved]
**Date Evaluated:** YYYY-MM-DD
**Evaluated By:** [Name/Team]
**Current Alternative:** [What we use instead]

## Executive Summary
Brief overview and recommendation

## What is [Feature]?
Technical explanation

## Current State
Benchmarks, measurements, status quo

## Pros and Cons Analysis
### Pros
### Cons

## Recommendation
Clear decision with rationale

## When to Revisit
Specific conditions for reconsidering

## Implementation Spec (If Reconsidered)
Detailed requirements and tasks

## References
Links to papers, discussions, related docs

## Appendix
Supporting data, benchmark details
```

---

## Related Documentation

- [Architecture - tech.md](../../.spec-workflow/steering/tech.md) - Technology decisions
- [Specs](../../.spec-workflow/specs/) - Implemented features
- [Research](../research/) - Technical investigations

---

## Contributing

When proposing a feature that gets deferred:

1. Create a document in this directory using the template above
2. Include benchmarks or measurements if applicable
3. Provide clear conditions for revisiting the decision
4. Update this README with a summary
5. Link to related discussions or issues

This ensures future contributors understand **why** decisions were made and **when** to reconsider them.

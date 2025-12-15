# KeyFlow Deployment Guide

## Overview

This document provides the deployment checklist and procedures for rolling out the KeyFlow feature to production. KeyFlow is a visual configuration flow editor that allows users to visualize and manage device-to-profile-to-keymap mappings.

## Pre-Deployment Checklist

### 1. Code Quality Gates

#### 1.1 All Tests Passing
- [ ] All KeyFlow unit tests passing (`flutter test test/repositories/keyflow_repository_test.dart`)
- [ ] All KeyFlow widget tests passing (`flutter test test/widgets/flow/`)
- [ ] All KeyFlow golden tests passing (`flutter test test/golden/flow_*`)
- [ ] All KeyFlow integration tests passing
- [ ] No flaky tests identified

#### 1.2 Test Coverage
- [ ] Overall KeyFlow coverage ≥80% verified
- [ ] Repository layer coverage ≥90%
- [ ] State management coverage ≥85%
- [ ] Widget tests for all interactive components
- [ ] Edge cases covered (empty states, errors, loading)

#### 1.3 Code Quality
- [ ] `flutter analyze` returns zero warnings/errors
- [ ] All files meet code metrics (≤500 lines/file, ≤50 lines/function)
- [ ] No TODO/FIXME comments in production code
- [ ] Code review completed and approved
- [ ] No hardcoded values or magic numbers
- [ ] All dependencies pinned to specific versions

### 2. Performance Benchmarks

#### 2.1 Rendering Performance
- [ ] 60fps maintained during pan/zoom operations
- [ ] 60fps maintained during node dragging
- [ ] Performance validated with 50+ devices (250+ nodes)
- [ ] Frame render time <16ms (verified with Flutter DevTools)
- [ ] No jank during animations or interactions

#### 2.2 Data Operations
- [ ] Initial page load <500ms (measured)
- [ ] Device update latency <100ms (measured)
- [ ] Save operation <200ms (measured)
- [ ] FFI call overhead acceptable (<50ms)
- [ ] Stream subscription overhead minimal

#### 2.3 Memory Usage
- [ ] Baseline memory usage <50MB (verified)
- [ ] Max memory usage <100MB with 50 devices (verified)
- [ ] No memory leaks detected (stream disposal, controller cleanup)
- [ ] Memory profiling completed with Flutter DevTools
- [ ] Stress test completed (100+ device connections/disconnections)

### 3. Accessibility Verification

#### 3.1 WCAG 2.1 AA Compliance
- [ ] All color contrasts meet 4.5:1 ratio for text
- [ ] All UI elements meet 3:1 ratio contrast
- [ ] Focus indicators visible and clear (2px outline)
- [ ] Touch targets ≥48x48px
- [ ] Text scalable without breaking layout

#### 3.2 Keyboard Navigation
- [ ] Tab navigation works through all nodes
- [ ] Arrow keys move focused node
- [ ] Enter opens node details
- [ ] Delete removes selected edge
- [ ] Escape deselects all
- [ ] Ctrl+0 resets zoom
- [ ] Ctrl+Plus/Minus zoom works
- [ ] No keyboard traps identified

#### 3.3 Screen Reader Support
- [ ] All nodes have semantic labels
- [ ] All edges have semantic labels
- [ ] State changes announced appropriately
- [ ] Tested with TalkBack (Android) or VoiceOver (iOS/macOS)
- [ ] All interactive elements accessible
- [ ] Error messages announced to screen readers

### 4. Functional Requirements Verification

#### 4.1 Data Fetching & Synchronization (FR1)
- [ ] RuntimeConfig fetched correctly via FFI
- [ ] Real-time device updates working (<100ms latency)
- [ ] Changes persist to RuntimeConfig correctly
- [ ] FFI error handling works (graceful degradation)
- [ ] Repository cache invalidation working

#### 4.2 Interactive Node Manipulation (FR2)
- [ ] Node dragging smooth (60fps)
- [ ] Profile slot reordering functional
- [ ] Add new slot dialog working
- [ ] Delete slot with confirmation working
- [ ] Toggle active/inactive working
- [ ] Position persistence across sessions

#### 4.3 Visual Encoding & Feedback (FR3)
- [ ] Node status colors correct (green/blue/red/grey)
- [ ] Edge visual encoding correct (solid/dashed)
- [ ] Priority indicators displayed on edges
- [ ] Validation errors shown with tooltips
- [ ] All visual states implemented

#### 4.4 Keyboard Shortcuts & Accessibility (FR4)
- [ ] All keyboard shortcuts functional
- [ ] Zoom and pan controls working
- [ ] Screen reader support verified
- [ ] Focus management working correctly

#### 4.5 Performance Requirements (FR5)
- [ ] 60fps rendering with 250+ nodes
- [ ] <500ms initial load
- [ ] <100ms device updates
- [ ] <200ms save operations
- [ ] <100MB memory with 50 devices

### 5. User Story Acceptance Criteria

#### US1: Visualize Current Configuration
- [ ] KeyFlow page shows all connected devices
- [ ] Active profiles and keymaps displayed
- [ ] Fallback slots distinguished (dashed lines)
- [ ] Node positions customizable and persist

#### US2: Reorder Fallback Priority
- [ ] Drag-drop slot reordering works
- [ ] Priority numbers update automatically
- [ ] Changes persist to RuntimeConfig immediately
- [ ] Visual feedback during reordering

#### US3: Add New Profile Slot
- [ ] "Add Slot" button accessible on device nodes
- [ ] Dialog shows available profiles and keymaps
- [ ] New slot appears with correct priority
- [ ] Form validation prevents invalid combinations

#### US4: Debug Configuration Errors
- [ ] Invalid nodes show red color
- [ ] Error icon with tooltip explaining issue
- [ ] Clicking error navigates to fix location
- [ ] All validation rules implemented

#### US5: Real-Time Device Updates
- [ ] New devices appear within 1s of connection
- [ ] Disconnected devices fade out
- [ ] No manual refresh required
- [ ] Animations smooth and non-jarring

### 6. Documentation Completeness

#### 6.1 User Documentation
- [ ] USER_GUIDE_KEYFLOW.md created and complete
- [ ] Screenshots included in documentation
- [ ] Common troubleshooting scenarios documented
- [ ] Keyboard shortcuts documented
- [ ] Onboarding dialog implemented

#### 6.2 Developer Documentation
- [ ] API documentation for KeyFlowRepository
- [ ] State management patterns documented
- [ ] FFI integration documented
- [ ] Testing guide for KeyFlow components
- [ ] Performance profiling guide created

#### 6.3 Deployment Documentation
- [ ] This deployment checklist complete
- [ ] Rollback procedure documented
- [ ] Feature flag configuration documented
- [ ] Monitoring and alerting setup documented

### 7. Observability & Monitoring

#### 7.1 Analytics Instrumentation
- [ ] Page view events tracked (keyflow_page_view)
- [ ] Interaction events tracked (slot_added, slot_deleted, etc.)
- [ ] Error events tracked with context
- [ ] Performance metrics instrumented (load time, interaction latency)
- [ ] Analytics tested and verified in staging

#### 7.2 Error Tracking
- [ ] FFI errors reported to error tracking service
- [ ] Validation failures logged with context
- [ ] Error tracking tested in staging
- [ ] Alert thresholds configured
- [ ] On-call procedures updated

#### 7.3 Structured Logging
- [ ] All KeyFlow operations logged in JSON format
- [ ] Log levels appropriate (DEBUG for dev, INFO for prod)
- [ ] No PII/secrets in logs
- [ ] Log aggregation configured
- [ ] Log retention policy applied

### 8. Feature Flag Configuration

#### 8.1 Feature Flag Setup
- [ ] `keyFlowEnabled` feature flag created
- [ ] Default value set to `false` for gradual rollout
- [ ] Flag tested in development environment
- [ ] Flag tested in staging environment
- [ ] Rollback procedure using flag verified

#### 8.2 Gradual Rollout Plan
- [ ] Phase 1: Internal testing (flag enabled for dev team)
- [ ] Phase 2: Beta users (10% rollout)
- [ ] Phase 3: Gradual ramp (25% → 50% → 100%)
- [ ] Rollback criteria defined
- [ ] Success metrics defined for each phase

### 9. Platform Testing

#### 9.1 Desktop Testing
- [ ] Tested on macOS (latest)
- [ ] Tested on Windows 10/11
- [ ] Tested on Linux (Ubuntu LTS)
- [ ] All features functional on desktop
- [ ] Performance acceptable on desktop

#### 9.2 Responsive Design
- [ ] Desktop layout verified (full-featured)
- [ ] Tablet layout verified (touch-friendly)
- [ ] Mobile layout verified (read-only or limited editing)
- [ ] Breakpoints tested

### 10. Integration Testing

#### 10.1 FFI Integration
- [ ] KeyFlowRepository → FFI bridge integration tested
- [ ] Device stream updates working correctly
- [ ] RuntimeConfig mutations persisted correctly
- [ ] Error handling for FFI failures verified
- [ ] FFI breaking changes process verified

#### 10.2 Cross-Feature Integration
- [ ] KeyFlow integrates with device registry
- [ ] KeyFlow integrates with profile management
- [ ] KeyFlow integrates with keymap compilation
- [ ] Navigation to/from KeyFlow works
- [ ] No conflicts with other features

### 11. Security Review

#### 11.1 Input Validation
- [ ] All user inputs validated
- [ ] No injection vulnerabilities (XSS, command injection)
- [ ] File path inputs sanitized
- [ ] FFI inputs validated before crossing boundary

#### 11.2 Data Privacy
- [ ] No PII logged or tracked
- [ ] Analytics respect privacy settings
- [ ] Data retention policies applied
- [ ] GDPR compliance verified (if applicable)

### 12. Backward Compatibility

#### 12.1 Data Compatibility
- [ ] Existing RuntimeConfig files load correctly
- [ ] Node positions gracefully handle missing data
- [ ] Old feature flag states handled correctly
- [ ] Migration path tested (if applicable)

#### 12.2 API Compatibility
- [ ] FFI bridge version compatibility verified
- [ ] No breaking changes to shared models
- [ ] Deprecation warnings added (if applicable)

## Deployment Procedure

### Phase 1: Pre-Production Validation

1. **Run Full CI Pipeline**
   ```bash
   # Ensure all quality gates pass
   gh workflow run keyflow-ci.yml
   gh workflow run flutter-ci.yml
   ```

2. **Manual Testing in Staging**
   - Load staging environment with feature flag enabled
   - Test all user stories end-to-end
   - Verify performance with realistic data (10+ devices)
   - Test error scenarios (disconnected devices, invalid configs)

3. **Performance Profiling**
   ```bash
   cd ui
   flutter run --profile
   # Use Flutter DevTools to verify:
   # - 60fps during interactions
   # - <100MB memory usage
   # - No memory leaks
   ```

4. **Accessibility Verification**
   - Enable screen reader and navigate entire flow
   - Use keyboard only (disconnect mouse)
   - Verify color contrast with automated tools

### Phase 2: Production Deployment

#### Step 1: Deploy Code (Feature Flag Disabled)

```bash
# Build production release
cd ui
flutter build web --release
flutter build macos --release
flutter build windows --release
flutter build linux --release

# Deploy to production servers
# (Follow existing deployment procedures)
```

#### Step 2: Enable Feature Flag for Internal Users

```bash
# Update feature flag configuration
# Set keyFlowEnabled = true for internal user accounts
# Monitor error rates and performance metrics
```

#### Step 3: Gradual Rollout to Beta Users (10%)

```bash
# Enable feature flag for 10% of users
# Monitor for 24 hours:
# - Error rate should be <1%
# - Page load time <500ms p95
# - No crash reports related to KeyFlow
```

#### Step 4: Increase Rollout (25% → 50% → 100%)

```bash
# Gradually increase rollout percentage
# At each stage, monitor for 24-48 hours:
# - User feedback (support tickets, surveys)
# - Analytics metrics (engagement, errors)
# - Performance metrics (load times, memory)
```

### Phase 3: Post-Deployment Monitoring

#### First 24 Hours

- [ ] Monitor error tracking dashboard for KeyFlow errors
- [ ] Check analytics for unexpected patterns
- [ ] Review performance metrics (load time, memory)
- [ ] Monitor user feedback channels
- [ ] Verify no increase in crash rate

#### First Week

- [ ] Review user engagement metrics
- [ ] Analyze user feedback and support tickets
- [ ] Identify and prioritize bug fixes
- [ ] Plan follow-up improvements
- [ ] Update documentation based on feedback

#### First Month

- [ ] Measure success against goals (see Success Metrics below)
- [ ] Conduct user interviews for qualitative feedback
- [ ] Plan next iteration features
- [ ] Update roadmap based on learnings

## Rollback Procedure

### Immediate Rollback (Emergency)

If critical issues are detected:

1. **Disable Feature Flag**
   ```bash
   # Set keyFlowEnabled = false globally
   # Users will no longer see KeyFlow navigation
   # Takes effect immediately (no redeploy needed)
   ```

2. **Verify Rollback**
   - Confirm KeyFlow page is no longer accessible
   - Verify existing functionality unaffected
   - Check error rates return to baseline

3. **Investigate Issue**
   - Review error logs and crash reports
   - Reproduce issue in staging environment
   - Identify root cause and fix

### Partial Rollback

If issues affect subset of users:

1. **Reduce rollout percentage** (e.g., 50% → 10%)
2. **Exclude affected user segments** (e.g., specific platform)
3. **Monitor and investigate** while maintaining partial availability

### Code Rollback

If feature flag rollback is insufficient:

```bash
# Revert to previous release
git revert <commit-hash>
# Follow standard deployment procedure to deploy reverted version
```

## Success Metrics

Track these metrics to measure KeyFlow success:

### Usage Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Weekly Active Users (WAU) | 50% of total users | Analytics: unique users visiting KeyFlow page |
| Average Session Duration | >2 minutes | Analytics: time spent on KeyFlow page |
| Slot Management Actions | >100/week | Analytics: slot_added + slot_deleted + slot_reordered events |
| Node Dragging Usage | >50/week | Analytics: node_dragged events |

### Quality Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Error Rate | <1% of sessions | Error tracking: errors per session |
| Page Load Time (p95) | <500ms | Performance monitoring: page load time |
| Interaction Latency (p95) | <100ms | Performance monitoring: interaction latency |
| Crash Rate | <0.1% | Crash reporting: KeyFlow-related crashes |

### User Satisfaction

| Metric | Target | How to Measure |
|--------|--------|----------------|
| User Satisfaction Score | >4/5 | User survey after 2 weeks of usage |
| Support Tickets | <5/week | Support system: KeyFlow-related tickets |
| Feature Adoption | >60% after 1 month | Analytics: % of users who use KeyFlow |

### Performance Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Rendering FPS | ≥60fps | Performance monitoring: frame render time |
| Memory Usage (p95) | <100MB | Performance monitoring: memory usage |
| Device Update Latency | <100ms | Performance monitoring: time from FFI event to UI update |

## Troubleshooting Common Issues

### Issue: KeyFlow page not loading

**Symptoms**: Blank page or infinite loading spinner

**Resolution**:
1. Check FFI bridge connection: `RuntimeService.getRuntimeConfig()`
2. Verify Flutter console for errors
3. Check network/FFI logs for failures
4. Fallback: Show error message with retry button

### Issue: Nodes not updating in real-time

**Symptoms**: Device connections don't appear automatically

**Resolution**:
1. Verify device stream subscription active
2. Check FFI bridge state update stream
3. Verify Riverpod provider refresh logic
4. Temporary workaround: Manual refresh button

### Issue: Performance degradation with many devices

**Symptoms**: UI lag, dropped frames, high memory usage

**Resolution**:
1. Profile with Flutter DevTools Timeline
2. Check RepaintBoundary usage
3. Verify virtualization for off-screen nodes
4. Optimize CustomPainter shouldRepaint logic
5. Consider pagination/filtering for 50+ devices

### Issue: Save failures

**Symptoms**: Changes not persisting to RuntimeConfig

**Resolution**:
1. Check FFI call logs for errors
2. Verify RuntimeConfig validation
3. Check file system permissions
4. Implement retry logic with exponential backoff
5. Show clear error message to user

## Release Notes Template

```markdown
# KeyFlow Feature Release

## What's New

**KeyFlow Visual Editor** is now available! Visualize and manage your keyboard configuration with an interactive node-based interface.

### Key Features

- 🎨 **Visual Configuration**: See your device-to-profile-to-keymap mappings at a glance
- 🔄 **Real-Time Updates**: Devices appear/disappear automatically when connected/disconnected
- 🎯 **Drag-and-Drop**: Reorder fallback priorities by dragging connections
- ➕ **Easy Slot Management**: Add/delete profile slots with intuitive dialogs
- ✅ **Validation Feedback**: Invalid configurations highlighted with helpful error messages
- ⌨️ **Keyboard Accessible**: Full keyboard navigation and screen reader support
- ⚡ **Auto-Save**: Changes persist automatically, no manual save required

### How to Access

Navigate to the KeyFlow page from the main navigation menu. On first visit, you'll see an onboarding dialog explaining the key concepts.

### Documentation

See [USER_GUIDE_KEYFLOW.md](docs/USER_GUIDE_KEYFLOW.md) for detailed usage instructions, keyboard shortcuts, and troubleshooting tips.

### Feedback

We'd love to hear your feedback! Please report issues at [GitHub Issues](https://github.com/your-org/keyrx/issues) or share your experience in our community forum.
```

## Contacts & Escalation

- **Feature Owner**: [Name/Team]
- **On-Call Engineer**: [Rotation/Contact]
- **Error Tracking Dashboard**: [Link]
- **Analytics Dashboard**: [Link]
- **Performance Monitoring**: [Link]

## Approval Sign-Off

- [ ] **Engineering Lead**: All code quality gates passed
- [ ] **QA Lead**: All acceptance criteria verified
- [ ] **Product Owner**: User stories implemented as specified
- [ ] **Security Team**: Security review completed
- [ ] **DevOps Lead**: Deployment procedure reviewed
- [ ] **Engineering Manager**: Ready for production deployment

---

**Last Updated**: [Date]
**Deployment Target**: [Production/Staging]
**Version**: [v1.0.0]

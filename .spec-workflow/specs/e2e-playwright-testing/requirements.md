# Requirements: E2E Playwright Testing

## Overview

Comprehensive end-to-end testing using Playwright to test all UI pages, API endpoints, and user flows. Prevent UAT errors by catching issues in automated tests before manual testing.

## Problem Statement

Current state:
- Schema mismatches found during UAT (devices, profiles)
- Rapid network requests from UI bugs
- No automated verification of all pages and endpoints
- Manual UAT is time-consuming and error-prone

Goal: Automated E2E tests that catch these issues before they reach UAT.

## User Stories

### US-1: All pages load without errors
**As a** developer
**I want** automated tests that verify all pages load correctly
**So that** broken pages are caught before UAT

**Acceptance Criteria:**
- EARS: When Playwright tests run, the system SHALL navigate to each page and verify no console errors or network failures
- All 6 UI pages tested: Home, Devices, Profiles, Config, Metrics, Simulator
- Tests verify page renders expected content

### US-2: All API endpoints respond correctly
**As a** developer
**I want** automated tests that verify all API endpoints return valid responses
**So that** API contract issues are caught before UAT

**Acceptance Criteria:**
- EARS: When Playwright tests run, the system SHALL test all documented API endpoints
- Tests verify response status codes (200, 201, 400, 404)
- Tests verify response body structure matches expected schema

### US-3: Critical user flows work end-to-end
**As a** developer
**I want** automated tests that verify critical user journeys
**So that** functional regressions are caught before UAT

**Acceptance Criteria:**
- EARS: When Playwright tests run, the system SHALL execute critical user flows
- Flows tested: Create profile, Activate profile, Edit config, View devices, View metrics

### US-4: Network requests are efficient
**As a** developer
**I want** automated tests that detect excessive network requests
**So that** performance issues like rapid PATCH requests are caught

**Acceptance Criteria:**
- EARS: When a page loads, the system SHALL NOT make more than expected number of API calls
- Tests detect duplicate/rapid requests to same endpoint
- Tests fail if unexpected request patterns occur

## Requirements

### Req 1: Page Load Tests
- 1.1: Test HomePage loads with dashboard content
- 1.2: Test DevicesPage loads device list
- 1.3: Test ProfilesPage loads profile list
- 1.4: Test ConfigPage loads editor
- 1.5: Test MetricsPage loads metrics dashboard
- 1.6: Test SimulatorPage loads simulator interface
- 1.7: Verify no JavaScript console errors on any page
- 1.8: Verify no failed network requests on any page

### Req 2: API Endpoint Tests
- 2.1: Test GET /api/status returns valid status
- 2.2: Test GET /api/devices returns device list
- 2.3: Test GET /api/profiles returns profile list
- 2.4: Test GET /api/profiles/:name/config returns config
- 2.5: Test POST /api/profiles creates new profile
- 2.6: Test POST /api/profiles/:name/activate activates profile
- 2.7: Test PUT /api/profiles/:name/config updates config
- 2.8: Test DELETE /api/profiles/:name deletes profile
- 2.9: Test PATCH /api/devices/:id updates device
- 2.10: Test GET /api/metrics/latency returns stats
- 2.11: Test GET /api/layouts returns layout list

### Req 3: User Flow Tests
- 3.1: Test profile creation flow (name → create → verify in list)
- 3.2: Test profile activation flow (select → activate → verify active)
- 3.3: Test config editing flow (open → edit → save → verify saved)
- 3.4: Test device layout change flow (select device → change layout → verify)
- 3.5: Test navigation flow (all sidebar/nav links work)

### Req 4: Network Efficiency Tests
- 4.1: Detect rapid duplicate requests to same endpoint
- 4.2: Verify page load makes expected number of requests
- 4.3: Verify user actions don't trigger unexpected requests
- 4.4: Log and fail on excessive API calls (>10 of same type per page)

### Req 5: CI Integration
- 5.1: Tests run in GitHub Actions CI
- 5.2: Tests run against daemon started in CI
- 5.3: Test results uploaded as artifacts
- 5.4: Screenshots captured on failure
- 5.5: Video recording for debugging

## API Endpoints to Test

| Method | Endpoint | Expected Status |
|--------|----------|-----------------|
| GET | /api/status | 200 |
| GET | /api/health | 200 |
| GET | /api/version | 200 |
| GET | /api/devices | 200 |
| PATCH | /api/devices/:id | 200 |
| PUT | /api/devices/:id/name | 200 |
| PUT | /api/devices/:id/layout | 200 |
| GET | /api/devices/:id/layout | 200 |
| DELETE | /api/devices/:id | 200 |
| GET | /api/profiles | 200 |
| POST | /api/profiles | 201 |
| GET | /api/profiles/active | 200 |
| POST | /api/profiles/:name/activate | 200 |
| GET | /api/profiles/:name/config | 200 |
| PUT | /api/profiles/:name/config | 200 |
| DELETE | /api/profiles/:name | 200 |
| POST | /api/profiles/:name/duplicate | 201 |
| PUT | /api/profiles/:name/rename | 200 |
| POST | /api/profiles/:name/validate | 200 |
| GET | /api/metrics/latency | 200 |
| GET | /api/metrics/events | 200 |
| DELETE | /api/metrics/events | 200 |
| GET | /api/daemon/state | 200 |
| GET | /api/config | 200 |
| PUT | /api/config | 200 |
| GET | /api/layers | 200 |
| GET | /api/layouts | 200 |
| GET | /api/layouts/:name | 200 |
| POST | /api/simulator/events | 200 |
| POST | /api/simulator/reset | 200 |

## UI Pages to Test

| Path | Page | Key Elements |
|------|------|--------------|
| /home | HomePage | Active profile card, device list, quick stats |
| /devices | DevicesPage | Device cards, layout selectors, rename buttons |
| /profiles | ProfilesPage | Profile list, create button, activate button |
| /profiles/:name/config | ConfigPage | Monaco editor, save button, profile selector |
| /config | ConfigPage | Monaco editor, save button |
| /metrics | MetricsPage | Latency stats, event log |
| /simulator | SimulatorPage | Key input, simulation output |

## Out of Scope

- WebSocket real-time message testing (separate spec)
- Performance/load testing (separate spec)
- Visual regression testing (existing visual tests)
- Mobile-specific testing (responsive tests exist)

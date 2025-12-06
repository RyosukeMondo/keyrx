# Tasks Document

- [x] 1. Relabel Editor navigation to Profiles and align page title
  - File: ui/lib/main.dart
  - Update the NavigationRail destination label from "Editor" to "Profiles"; ensure any app bar/title text in the editor screen reflects Profiles without changing the underlying route/widget wiring.
  - _Leverage: ui/lib/main.dart_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Relabel the Editor navigation item to Profiles and keep routing intact | Restrictions: Do not change route keys/indices; avoid altering other destinations | Success: Navigation shows Profiles where Editor was, and selecting it still opens the same screen._

- [ ] 2. Add storage path resolver for home `.keyrx`
  - File: ui/lib/services/storage_path_resolver.dart (new)
  - Create a service that resolves the profiles directory to `%USERPROFILE%/.keyrx` on Windows and `~/.keyrx` on Linux, ensures the directory exists, and exposes the resolved path.
  - _Leverage: dart:io Platform APIs; existing service patterns in ui/lib/services/*
  - _Requirements: 1, 4_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Build a storage path resolver that returns/creates the user home .keyrx directory cross-platform | Restrictions: No elevated permissions; do not hardcode absolute user names | Success: Service returns a valid directory path, creates it if missing, handles Windows and Linux._

- [ ] 3. Implement profile autosave service (debounce + retry)
  - File: ui/lib/services/profile_autosave_service.dart (new)
  - Add a debounced autosave helper that writes profiles via the profile repository to the resolved `.keyrx` path, emits saving/success/error status, and retries up to 3 times on transient failures with backoff.
  - _Leverage: ui/lib/services/storage_path_resolver.dart (task 2), existing profile repository/service in ui/lib/services and ui/lib/repositories_
  - _Requirements: 1, 4, Reliability_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart platform engineer | Task: Add a debounced autosave service with retry that persists profiles to the .keyrx path and surfaces status | Restrictions: Keep UI thread non-blocking; avoid duplicate writes by debouncing; follow existing service patterns | Success: Autosave can be triggered with profile data, writes to .keyrx, reports status, retries on transient errors._

- [ ] 4. Improve Profiles layout UX (responsive preview and validation)
  - Files: ui/lib/pages/visual_editor_page.dart; ui/lib/pages/visual_editor_widgets.dart
  - Make the layout setup step clearer: enforce minimum height/width for the preview to avoid narrow stacking, add validation for rows/cols per row, and adjust spacing/typography for readability.
  - _Leverage: existing layout editor and grid preview widgets in visual_editor_page.dart and visual_editor_widgets.dart_
  - _Requirements: 3, 5_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI/UX engineer | Task: Refine the layout setup UI with validation, responsive preview sizing, and clearer spacing to prevent overly narrow vertical sections | Restrictions: Reuse existing widgets; avoid breaking current interactions | Success: Layout editor stays readable at typical sizes, invalid inputs are blocked with inline messaging, preview scales without vertical squish._

- [ ] 5. Create Mapping page for per-key assignment
  - Files: ui/lib/pages/mapping_page.dart (new); ui/lib/widgets/mapping_grid.dart (new, if needed)
  - Build a new Mapping screen that renders the profile grid with consistent cell sizing, supports density toggle (comfortable/compact), search/filter highlights, and invokes the mapping editor panel/dialog for key/action assignment.
  - _Leverage: grid/rendering patterns from visual_editor_widgets.dart; mapping/action editor components already used in the editor if present_
  - _Requirements: 4, 5_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer specializing in complex grids | Task: Add a Mapping page with grid view, density toggle, search/filter highlighting, and mapping editor invocation | Restrictions: Keep rendering performant; reuse existing grid/key widgets where possible; maintain accessibility (focus/keyboard nav) | Success: Mapping page is reachable, shows the grid, supports density toggle and search highlight, and opens the mapping editor for a cell._

- [ ] 6. Wire navigation to include Mapping and rename Editor to Profiles
  - Files: ui/lib/main.dart; any nav drawer/rail configs
  - Add the Mapping destination and page to the navigation (Rail and page list), and ensure the existing Editor label is now Profiles across nav/drawer/status surfaces.
  - _Leverage: ui/lib/main.dart navigation destinations and page construction_
  - _Requirements: 1, 4_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter navigation engineer | Task: Add Mapping as a new destination and update Editor label to Profiles without breaking indices/routes | Restrictions: Preserve existing order except where adding the new Mapping entry; keep selectedIndex wiring intact | Success: Navigation shows Profiles and Mapping; selecting each shows the correct page._

- [ ] 7. Enhance Devices page for discovery and friendly naming
  - File: ui/lib/pages/devices_page.dart
  - Add empty state messaging, inline/dialog rename with immediate persistence, and a non-blocking discovery indicator; display vendor/product ids with friendly name or placeholder.
  - _Leverage: existing device discovery and list UI in devices_page.dart_
  - _Requirements: 2, 5_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Improve Devices page with empty/loading states and inline rename that persists immediately | Restrictions: Keep discovery non-blocking; reuse current list patterns; avoid breaking device selection | Success: Devices page shows friendly names with ids, supports rename, shows empty/refresh guidance, and shows discovery progress unobtrusively._

- [ ] 8. Hook autosave into Profiles and Mapping flows with feedback
  - Files: ui/lib/pages/visual_editor_page.dart; ui/lib/pages/mapping_page.dart
  - Invoke autosave on profile/mapping changes using the autosave service; surface non-blocking status (saving/last saved/error) via toast/banner; ensure in-progress saves are debounced and last-writer wins.
  - _Leverage: profile_autosave_service.dart (task 3), storage_path_resolver.dart (task 2), existing toast/snackbar patterns_
  - _Requirements: 1, 4, Reliability, Usability_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter integration engineer | Task: Wire autosave into profile/mapping edits with status feedback and debounce | Restrictions: Non-blocking UI; avoid duplicate toasts; ensure errors are shown without losing in-memory edits | Success: Edits trigger debounced autosave to .keyrx with visible status and retries on failure._

- [ ] 9. Add tests for storage path resolver and autosave debounce/retry
  - Files: ui/test/storage_path_resolver_test.dart; ui/test/profile_autosave_service_test.dart
  - Add unit tests covering Windows vs Linux path resolution, directory creation, debounce behavior, and retry on transient save errors.
  - _Leverage: existing test harness in ui/test; dart:io mocks or test doubles_
  - _Requirements: Reliability, Performance_
  - _Prompt: Implement the task for spec mapping-screen-refresh, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart test engineer | Task: Add unit tests for storage path resolver and autosave debounce/retry logic | Restrictions: Avoid flaky timing; use fake timers/mocks where possible | Success: Tests pass, cover success/error/retry paths, and validate Windows/Linux path outputs._

# Design Document

## Architecture

```
UI Profile List → REST API → Daemon Profile Manager → Config Loader
                                  ↓
                          ~/.config/keyrx/profiles/
                              ├── work.krx
                              ├── gaming.krx
                              └── coding.krx
```

## Components

### 1. Profile Manager (Rust - keyrx_daemon/src/profile_manager.rs)
- CRUD operations for profiles
- Load profile and apply to daemon

### 2. REST API Endpoints (keyrx_daemon/src/web/api.rs)
```
GET    /api/profiles          - List all profiles
POST   /api/profiles          - Create new profile
PUT    /api/profiles/:id      - Update profile
DELETE /api/profiles/:id      - Delete profile
POST   /api/profiles/:id/activate - Activate profile
```

### 3. ProfilesPage (keyrx_ui/src/pages/ProfilesPage.tsx)

**UI Layout:**
```
+---------------------------------------------------------------+
| Profiles                                   [+ New Profile]    |
+---------------------------------------------------------------+
| [●] work                              Last modified: 2d ago   |
|     Quick access for work shortcuts              [Activate]  |
|     [Rename] [Duplicate] [Export] [Delete]                   |
+---------------------------------------------------------------+
| [ ] gaming                            Last modified: 5d ago   |
|     Optimized for FPS games                      [Activate]  |
|     [Rename] [Duplicate] [Export] [Delete]                   |
+---------------------------------------------------------------+
| [ ] coding                            Last modified: 1w ago   |
|     VSCode and terminal shortcuts                [Activate]  |
|     [Rename] [Duplicate] [Export] [Delete]                   |
+---------------------------------------------------------------+
```

### 4. ProfileCard (keyrx_ui/src/components/ProfileCard.tsx)
- Display profile info with action buttons
- Hover to show actions

### 5. ProfileDialog (keyrx_ui/src/components/ProfileDialog.tsx)
- Create/rename profile modal
- Name and description inputs

## Data Models

```typescript
interface Profile {
  id: string;
  name: string;
  description: string;
  created_at: number;
  modified_at: number;
  is_default: boolean;
  is_active: boolean;
  config_path: string;  // Path to .krx file
}

interface ProfileMetadata {
  name: string;
  description: string;
  created_at: number;
  modified_at: number;
}
```

## Dependencies

- No new UI dependencies (use existing React)

## Sources

- [Designing profile pages for better UX](https://medium.com/design-bootcamp/designing-profile-account-and-setting-pages-for-better-ux-345ef4ca1490)

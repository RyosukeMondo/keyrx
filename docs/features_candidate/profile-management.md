# Profile Management

Profile management in KeyRX allows you to maintain multiple keyboard configuration profiles and switch between them seamlessly. Each profile is a complete keyboard remapping configuration with its own layers, key mappings, and macro definitions.

## Overview

The profile system provides:

- **Multiple Configurations**: Create and maintain separate profiles for different use cases (gaming, programming, general use, etc.)
- **Quick Switching**: Activate any profile with a single click
- **Configuration Backup**: Export and import profiles for backup or sharing
- **Template System**: Start new profiles from blank or QMK-compatible templates
- **Profile Duplication**: Clone existing profiles to create variations

## Getting Started

### Prerequisites

- KeyRX daemon running (`keyrx_daemon`)
- KeyRX UI accessible (default: `http://localhost:5173`)
- Or use CLI commands directly

### Accessing Profile Management

#### Web UI

1. Open the KeyRX UI in your browser
2. Click the **Profiles** button in the navigation bar
3. The profile management page will display all available profiles

#### CLI

```bash
# List all profiles
keyrx_daemon profiles list

# Or with JSON output
keyrx_daemon profiles list --json
```

## Creating Profiles

### Using the Web UI

1. Click the **+ New Profile** button
2. Enter a profile name (alphanumeric, dash, underscore only)
3. Select a template:
   - **Blank**: Empty profile with no mappings
   - **QMK**: QMK-compatible layer structure
4. Click **Create**

The new profile will appear in the profile list.

### Using the CLI

```bash
# Create a blank profile
keyrx_daemon profiles create my-profile

# Create a profile from QMK template
keyrx_daemon profiles create gaming-profile --template qmk
```

### Profile Naming Rules

Profile names must:
- Contain only alphanumeric characters, dashes (`-`), and underscores (`_`)
- Be unique (no duplicate names)
- Be between 1 and 64 characters long

**Valid names**: `gaming`, `work-setup`, `programming_config`, `profile-2024`

**Invalid names**: `my profile` (spaces), `test@home` (special chars), `` (empty)

## Managing Profiles

### Viewing Profile Information

Each profile card displays:
- **Profile Name**: The unique identifier
- **Layer Count**: Number of layers defined in the profile
- **Modified Date**: Last modification timestamp
- **Active Status**: Visual indicator if this profile is currently active

### Activating a Profile

The active profile is the one currently being used by the KeyRX daemon for keyboard remapping.

#### Web UI

1. Find the profile you want to activate
2. Hover over the profile card to reveal actions
3. Click the **Activate** button
4. The profile will be activated immediately

Active profiles show a visual indicator (checkmark or highlight).

#### CLI

```bash
# Activate a profile by name
keyrx_daemon profiles activate my-profile

# Verify active profile
keyrx_daemon profiles list --json | jq '.profiles[] | select(.is_active==true)'
```

### Duplicating Profiles

Create a copy of an existing profile to use as a starting point for variations.

#### Web UI

1. Hover over the profile card
2. Click the **Duplicate** button
3. Enter a name for the duplicated profile
4. Click **OK**

The duplicated profile will appear in the list with the new name.

#### CLI

```bash
# Duplicate a profile
keyrx_daemon profiles duplicate gaming-profile gaming-profile-2
```

### Exporting Profiles

Export profiles to back up configurations or share them with others.

#### Web UI

1. Hover over the profile card
2. Click the **Export** button
3. The `.rhai` configuration file will download to your default downloads folder

The exported file contains the complete Rhai configuration for the profile.

#### CLI

```bash
# Export a profile to a specific file
keyrx_daemon profiles export my-profile /path/to/backup.rhai
```

### Importing Profiles

Import previously exported profiles or configurations from other sources.

#### CLI

```bash
# Import a profile from a .rhai file
keyrx_daemon profiles import new-profile /path/to/config.rhai
```

**Note**: Import functionality is currently CLI-only.

### Deleting Profiles

Remove profiles you no longer need.

#### Web UI

1. Hover over the profile card
2. Click the **Delete** button
3. Confirm the deletion in the dialog
4. The profile will be permanently removed

**Warning**: Profile deletion is permanent and cannot be undone. Export important profiles before deleting.

#### CLI

```bash
# Delete a profile
keyrx_daemon profiles delete old-profile
```

**Note**: You cannot delete the currently active profile. Activate a different profile first.

## Profile Limitations

- **Maximum Profiles**: 100 profiles per system
- **Profile Size**: Limited by available disk space
- **Active Profile**: Only one profile can be active at a time
- **Name Uniqueness**: Each profile must have a unique name

## File Structure

Profiles are stored in the system configuration directory:

```
~/.config/keyrx/profiles/
├── gaming/
│   ├── config.rhai       # Rhai source configuration
│   └── config.krx        # Compiled binary (auto-generated)
├── programming/
│   ├── config.rhai
│   └── config.krx
└── default/
    ├── config.rhai
    └── config.krx
```

Each profile directory contains:
- `config.rhai`: Human-readable Rhai configuration (edit this)
- `config.krx`: Compiled binary (auto-generated, do not edit)

## Best Practices

### Profile Organization

- **Specific Names**: Use descriptive names that indicate the profile's purpose
  - Good: `gaming-fps`, `work-coding`, `general-use`
  - Bad: `profile1`, `test`, `new`

- **Regular Exports**: Export important profiles regularly for backup
  ```bash
  keyrx_daemon profiles export gaming-fps ~/backups/gaming-fps-$(date +%Y%m%d).rhai
  ```

- **Test Before Activating**: Use the simulator to test profile changes before activation

### Configuration Workflow

1. **Duplicate** an existing profile as a starting point
2. **Edit** the `.rhai` file in the Config Editor
3. **Validate** the configuration with the validator
4. **Test** with the WASM simulator
5. **Activate** the profile when ready

### Version Control

For advanced users managing profiles with git:

```bash
cd ~/.config/keyrx/profiles
git init
git add .
git commit -m "Initial profiles"

# After changes
git commit -am "Update gaming profile with new macros"
```

## Troubleshooting

### Profile Won't Activate

**Symptoms**: Clicking "Activate" has no effect or shows an error.

**Solutions**:
1. Check daemon logs: `journalctl -u keyrx-daemon -f`
2. Verify the `.rhai` file is valid: Use Config Editor validator
3. Ensure the compiled `.krx` file exists: It should auto-generate
4. Check file permissions: Must be readable by the daemon

### Missing Profile

**Symptoms**: Profile doesn't appear in the list.

**Solutions**:
1. Check if the directory exists in `~/.config/keyrx/profiles/`
2. Verify the directory contains a `config.rhai` file
3. Check directory permissions
4. Reload the profiles list (click Profiles tab again)

### Cannot Create Profile

**Symptoms**: Create button is disabled or shows validation error.

**Solutions**:
1. Verify the profile name follows naming rules (alphanumeric, dash, underscore)
2. Check if a profile with that name already exists
3. Ensure you haven't reached the 100 profile limit
4. Verify disk space is available

### Export Download Fails

**Symptoms**: Export button doesn't trigger a download.

**Solutions**:
1. Check browser download settings (pop-up blockers, download location)
2. Verify the `.rhai` file exists: `ls ~/.config/keyrx/profiles/<name>/config.rhai`
3. Check file permissions
4. Try using CLI export instead

## API Reference

### REST API Endpoints

The KeyRX daemon exposes the following REST API endpoints for profile management:

#### List Profiles
```http
GET /api/profiles
```

**Response**:
```json
{
  "profiles": [
    {
      "name": "gaming",
      "rhai_path": "/home/user/.config/keyrx/profiles/gaming/config.rhai",
      "krx_path": "/home/user/.config/keyrx/profiles/gaming/config.krx",
      "modified_at": 1703980800,
      "layer_count": 3,
      "is_active": true
    }
  ]
}
```

#### Create Profile
```http
POST /api/profiles
Content-Type: application/json

{
  "name": "new-profile",
  "template": "blank"  // or "qmk"
}
```

**Response**: `201 Created` with profile details

#### Activate Profile
```http
POST /api/profiles/{name}/activate
```

**Response**: `200 OK` with success message

#### Delete Profile
```http
DELETE /api/profiles/{name}
```

**Response**: `200 OK` with success message

#### Duplicate Profile
```http
POST /api/profiles/{name}/duplicate
Content-Type: application/json

{
  "dest": "new-profile-name"
}
```

**Response**: `200 OK` with success message

### CLI Commands

```bash
# List all profiles
keyrx_daemon profiles list [--json]

# Create a new profile
keyrx_daemon profiles create <name> [--template blank|qmk]

# Activate a profile
keyrx_daemon profiles activate <name>

# Delete a profile
keyrx_daemon profiles delete <name>

# Duplicate a profile
keyrx_daemon profiles duplicate <source> <destination>

# Export a profile
keyrx_daemon profiles export <name> <output-path>

# Import a profile
keyrx_daemon profiles import <name> <input-path>
```

## Advanced Usage

### Automated Profile Switching

You can create scripts to automatically switch profiles based on the active application:

```bash
#!/bin/bash
# Switch to gaming profile when launching a game

if pgrep -x "game-binary" > /dev/null; then
    keyrx_daemon profiles activate gaming-fps
else
    keyrx_daemon profiles activate general-use
fi
```

### Shared Profiles

To share profiles across multiple machines:

1. Export the profile: `keyrx_daemon profiles export my-profile shared.rhai`
2. Copy `shared.rhai` to the other machine
3. Import on the other machine: `keyrx_daemon profiles import my-profile shared.rhai`

### Profile Templates

Create your own base profiles as templates:

1. Create a base profile with common settings
2. Export it: `keyrx_daemon profiles export base-template ~/templates/base.rhai`
3. For new profiles:
   ```bash
   keyrx_daemon profiles create new-profile
   keyrx_daemon profiles import new-profile ~/templates/base.rhai
   ```

## Related Features

- **Config Editor**: Edit profile `.rhai` files with syntax highlighting and validation
- **WASM Simulator**: Test profile configurations before activation
- **Macro Recorder**: Record and add macros to your profiles
- **Validation**: Real-time configuration validation and error checking

## See Also

- [Configuration Validation](./config-validation.md) - Validate profile configurations
- [Macro Recorder](./macro-recorder.md) - Create macros for profiles
- [WASM Simulation](./wasm-simulation.md) - Test profiles safely

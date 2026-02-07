#!/usr/bin/env python3
"""
KeyRx System Tray Application

Provides GNOME/KDE system tray integration for KeyRx daemon.
Controls daemon, switches profiles, and provides quick access to web UI.
"""

import gi
import os
import sys
import signal
import requests
import subprocess
import webbrowser
from typing import Optional

gi.require_version('Gtk', '3.0')
gi.require_version('AppIndicator3', '0.1')
gi.require_version('Notify', '0.7')

from gi.repository import Gtk, AppIndicator3, GLib, Notify

# Configuration
DAEMON_API_BASE = os.getenv('KEYRX_API_URL', 'http://127.0.0.1:9867')
WEB_UI_URL = os.getenv('KEYRX_WEB_UI', 'http://127.0.0.1:9867')
MOCK_MODE = os.getenv('KEYRX_MOCK', '0') == '1'
UPDATE_INTERVAL = 5000  # ms

class KeyRxTray:
    def __init__(self):
        # Initialize libnotify
        Notify.init("KeyRx")

        # Create indicator
        self.indicator = AppIndicator3.Indicator.new(
            "keyrx-indicator",
            "input-keyboard",  # Icon name from theme
            AppIndicator3.IndicatorCategory.HARDWARE
        )
        self.indicator.set_status(AppIndicator3.IndicatorStatus.ACTIVE)

        # State
        self.daemon_running = False
        self.current_profile = None
        self.remapping_enabled = True

        # Build menu
        self.menu = self.build_menu()
        self.indicator.set_menu(self.menu)

        # Start status polling
        GLib.timeout_add(UPDATE_INTERVAL, self.update_status)
        self.update_status()  # Initial update

    def build_menu(self):
        """Build system tray menu"""
        menu = Gtk.Menu()

        # Status item (non-clickable)
        self.status_item = Gtk.MenuItem(label="Status: Checking...")
        self.status_item.set_sensitive(False)
        menu.append(self.status_item)

        menu.append(Gtk.SeparatorMenuItem())

        # Enable/Disable toggle
        self.toggle_item = Gtk.CheckMenuItem(label="Enable Remapping")
        self.toggle_item.set_active(True)
        self.toggle_item.connect("activate", self.on_toggle_remapping)
        menu.append(self.toggle_item)

        menu.append(Gtk.SeparatorMenuItem())

        # Profiles submenu
        self.profiles_menu_item = Gtk.MenuItem(label="Switch Profile")
        self.profiles_submenu = Gtk.Menu()
        self.profiles_menu_item.set_submenu(self.profiles_submenu)
        menu.append(self.profiles_menu_item)

        # Refresh profiles
        refresh_profiles = Gtk.MenuItem(label="↻ Refresh Profiles")
        refresh_profiles.connect("activate", lambda _: self.update_profiles())
        self.profiles_submenu.append(refresh_profiles)
        self.profiles_submenu.append(Gtk.SeparatorMenuItem())

        menu.append(Gtk.SeparatorMenuItem())

        # Open Web UI
        web_ui_item = Gtk.MenuItem(label="Open Web UI")
        web_ui_item.connect("activate", self.on_open_web_ui)
        menu.append(web_ui_item)

        # Settings (opens web UI to settings page)
        settings_item = Gtk.MenuItem(label="Settings")
        settings_item.connect("activate", self.on_open_settings)
        menu.append(settings_item)

        menu.append(Gtk.SeparatorMenuItem())

        # About
        about_item = Gtk.MenuItem(label="About")
        about_item.connect("activate", self.on_about)
        menu.append(about_item)

        # Quit
        quit_item = Gtk.MenuItem(label="Quit Tray")
        quit_item.connect("activate", self.on_quit)
        menu.append(quit_item)

        menu.show_all()
        return menu

    def api_get(self, endpoint: str) -> Optional[dict]:
        """Make GET request to daemon API"""
        if MOCK_MODE:
            return self.mock_api_response(endpoint)

        try:
            response = requests.get(f"{DAEMON_API_BASE}{endpoint}", timeout=2)
            response.raise_for_status()
            return response.json()
        except Exception as e:
            print(f"API GET error: {e}", file=sys.stderr)
            return None

    def api_post(self, endpoint: str, data: dict = None) -> bool:
        """Make POST request to daemon API"""
        if MOCK_MODE:
            print(f"Mock POST {endpoint}: {data}")
            return True

        try:
            response = requests.post(
                f"{DAEMON_API_BASE}{endpoint}",
                json=data,
                timeout=2
            )
            response.raise_for_status()
            return True
        except Exception as e:
            print(f"API POST error: {e}", file=sys.stderr)
            return False

    def mock_api_response(self, endpoint: str) -> dict:
        """Mock API responses for testing without daemon"""
        if endpoint == "/api/status":
            return {
                "running": True,
                "version": "0.1.0",
                "profile": "default",
                "remapping_enabled": self.remapping_enabled
            }
        elif endpoint == "/api/profiles":
            return {
                "profiles": [
                    {"name": "default", "active": True},
                    {"name": "gaming", "active": False},
                    {"name": "coding", "active": False}
                ]
            }
        return {}

    def update_status(self) -> bool:
        """Update daemon status from API"""
        status = self.api_get("/api/status")

        if status:
            self.daemon_running = True
            self.current_profile = status.get("profile", "Unknown")
            enabled = status.get("remapping_enabled", True)

            # Update UI
            self.status_item.set_label(f"Profile: {self.current_profile}")
            self.toggle_item.set_active(enabled)
            self.indicator.set_icon("input-keyboard" if enabled else "input-keyboard-symbolic")

            # Update profiles list
            if not hasattr(self, '_profiles_loaded'):
                self.update_profiles()
                self._profiles_loaded = True
        else:
            self.daemon_running = False
            self.status_item.set_label("Status: Daemon not running")
            self.indicator.set_icon("input-keyboard-symbolic")

        return True  # Continue polling

    def update_profiles(self):
        """Refresh profiles submenu"""
        # Clear existing profile items (keep refresh button and separator)
        for item in self.profiles_submenu.get_children()[2:]:
            self.profiles_submenu.remove(item)

        profiles_data = self.api_get("/api/profiles")
        if not profiles_data:
            no_profiles = Gtk.MenuItem(label="No profiles available")
            no_profiles.set_sensitive(False)
            self.profiles_submenu.append(no_profiles)
        else:
            profiles = profiles_data.get("profiles", [])
            for profile in profiles:
                item = Gtk.CheckMenuItem(label=profile["name"])
                item.set_active(profile.get("active", False))
                item.connect("activate", self.on_switch_profile, profile["name"])
                self.profiles_submenu.append(item)

        self.profiles_submenu.show_all()

    def on_toggle_remapping(self, widget):
        """Toggle remapping on/off"""
        enabled = widget.get_active()
        self.remapping_enabled = enabled

        if self.api_post("/api/toggle", {"enabled": enabled}):
            status = "enabled" if enabled else "disabled"
            self.show_notification(
                "KeyRx Remapping",
                f"Remapping {status}",
                "input-keyboard"
            )
        else:
            # Revert on failure
            widget.set_active(not enabled)
            self.show_notification(
                "KeyRx Error",
                "Failed to toggle remapping",
                "dialog-error"
            )

    def on_switch_profile(self, widget, profile_name: str):
        """Switch to different profile"""
        if self.api_post("/api/profiles/activate", {"name": profile_name}):
            self.current_profile = profile_name
            self.show_notification(
                "KeyRx Profile",
                f"Switched to profile: {profile_name}",
                "input-keyboard"
            )
            self.update_profiles()
        else:
            self.show_notification(
                "KeyRx Error",
                f"Failed to switch to profile: {profile_name}",
                "dialog-error"
            )

    def on_open_web_ui(self, widget):
        """Open web UI in default browser"""
        webbrowser.open(WEB_UI_URL)

    def on_open_settings(self, widget):
        """Open settings page in web UI"""
        webbrowser.open(f"{WEB_UI_URL}/#/settings")

    def on_about(self, widget):
        """Show about dialog"""
        dialog = Gtk.AboutDialog()
        dialog.set_program_name("KeyRx")
        dialog.set_version("1.0.0")
        dialog.set_comments("Advanced keyboard remapping with layers and tap-hold")
        dialog.set_website("https://github.com/RyosukeMondo/keyrx")
        dialog.set_logo_icon_name("input-keyboard")
        dialog.set_authors(["KeyRx Contributors"])
        dialog.set_license_type(Gtk.License.AGPL_3_0)
        dialog.run()
        dialog.destroy()

    def on_quit(self, widget):
        """Quit tray application (daemon keeps running)"""
        Notify.uninit()
        Gtk.main_quit()

    def show_notification(self, title: str, message: str, icon: str = "input-keyboard"):
        """Show desktop notification"""
        notification = Notify.Notification.new(title, message, icon)
        notification.show()


def check_daemon_running() -> bool:
    """Check if daemon is accessible"""
    if MOCK_MODE:
        return True

    try:
        response = requests.get(f"{DAEMON_API_BASE}/api/status", timeout=2)
        return response.status_code == 200
    except:
        return False


def main():
    # Handle Ctrl+C gracefully
    signal.signal(signal.SIGINT, signal.SIG_DFL)

    # Check if daemon is running (warn but don't exit)
    if not check_daemon_running() and not MOCK_MODE:
        print("Warning: KeyRx daemon is not running", file=sys.stderr)
        print("Start daemon with: sudo systemctl start keyrx", file=sys.stderr)
        print("Tray will continue and retry connection...", file=sys.stderr)

    # Create tray
    tray = KeyRxTray()

    print("KeyRx system tray started")
    print(f"Daemon API: {DAEMON_API_BASE}")
    print(f"Web UI: {WEB_UI_URL}")

    # Run GTK main loop
    Gtk.main()


if __name__ == '__main__':
    main()

//! Single source for the "how do I get device access" advice text.
//!
//! `input`/`uinput` must exist as *system* groups (`groupadd -r`): udev is
//! deprecating device-node ownership by normal-GID groups, and the installer
//! creates them that way. Every user-facing hint goes through here so the
//! advice cannot drift between the CLI, the platform layer and the tests.

/// Shell command that creates `group` (if missing) and adds the current user to it.
pub fn join_group_command(group: &str) -> String {
    format!("sudo groupadd -r -f {group} && sudo usermod -aG {group} $USER")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_system_group_before_joining() {
        assert_eq!(
            join_group_command("uinput"),
            "sudo groupadd -r -f uinput && sudo usermod -aG uinput $USER"
        );
    }
}

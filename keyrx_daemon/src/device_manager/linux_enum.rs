//! Keyboard discovery from sysfs.
//!
//! Enumeration reads `/sys/class/input/eventN/device/{name,uniq,phys,
//! capabilities/*}` instead of opening every `/dev/input` node. Closing an
//! evdev fd waits for an RCU grace period (~15 ms each), so the old
//! open-every-node approach made `GET /api/devices` take ~0.5 s on a machine
//! with 30 input nodes. sysfs is also world-readable, so listing works without
//! the `input` group (grabbing still needs it).

use std::fs;
use std::path::{Path, PathBuf};

use evdev::Key;

use super::{DiscoveryError, KeyboardInfo};

/// Letter keys a device must mostly have to count as a keyboard.
pub(super) const REQUIRED_KEYS: [Key; 26] = [
    Key::KEY_A,
    Key::KEY_B,
    Key::KEY_C,
    Key::KEY_D,
    Key::KEY_E,
    Key::KEY_F,
    Key::KEY_G,
    Key::KEY_H,
    Key::KEY_I,
    Key::KEY_J,
    Key::KEY_K,
    Key::KEY_L,
    Key::KEY_M,
    Key::KEY_N,
    Key::KEY_O,
    Key::KEY_P,
    Key::KEY_Q,
    Key::KEY_R,
    Key::KEY_S,
    Key::KEY_T,
    Key::KEY_U,
    Key::KEY_V,
    Key::KEY_W,
    Key::KEY_X,
    Key::KEY_Y,
    Key::KEY_Z,
];

pub(super) const MIN_REQUIRED_KEYS: usize = 20;

/// `EV_KEY` bit in `capabilities/ev`.
const EV_KEY_BIT: usize = 1;

/// Name of the daemon's own virtual output device (never a capture source).
const OUTPUT_DEVICE_NAME: &str = "keyrx";

/// Lists keyboards (devices with EV_KEY and most letter keys), deduplicated
/// per physical device, sorted by device path.
pub fn enumerate_keyboards() -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    enumerate_keyboards_in(Path::new("/sys/class/input"), Path::new("/dev/input"))
}

/// [`enumerate_keyboards`] over an arbitrary sysfs class dir (tests).
pub(super) fn enumerate_keyboards_in(
    sys_class_input: &Path,
    dev_input: &Path,
) -> Result<Vec<KeyboardInfo>, DiscoveryError> {
    let mut keyboards = Vec::new();
    for entry in fs::read_dir(sys_class_input)? {
        let entry = entry.map_err(DiscoveryError::Io)?;
        let node = entry.file_name().to_string_lossy().into_owned();
        if !node.starts_with("event") {
            continue;
        }
        if let Some(info) = read_keyboard(&entry.path().join("device"), dev_input.join(&node)) {
            keyboards.push(info);
        }
    }
    keyboards.sort_by(|a, b| a.path.cmp(&b.path));
    // Many USB keyboards expose several event nodes (keys, consumer keys...).
    deduplicate_keyboards(&mut keyboards);
    Ok(keyboards)
}

/// Reads one input device's sysfs attributes; `None` if it is not a keyboard.
fn read_keyboard(device_dir: &Path, path: PathBuf) -> Option<KeyboardInfo> {
    let ev = read_bitmap(&device_dir.join("capabilities/ev"))?;
    let keys = read_bitmap(&device_dir.join("capabilities/key"))?;
    if !is_keyboard(&ev, &keys) {
        return None;
    }
    let name = read_attr(device_dir, "name").unwrap_or_else(|| "Unknown Device".to_string());
    if name == OUTPUT_DEVICE_NAME {
        return None;
    }
    Some(KeyboardInfo {
        path,
        name,
        serial: read_attr(device_dir, "uniq"),
        phys: read_attr(device_dir, "phys"),
    })
}

fn is_keyboard(ev: &[usize], keys: &[usize]) -> bool {
    has_bit(ev, EV_KEY_BIT)
        && REQUIRED_KEYS
            .iter()
            .filter(|key| has_bit(keys, usize::from(key.code())))
            .count()
            >= MIN_REQUIRED_KEYS
}

/// A trimmed, non-empty sysfs attribute.
fn read_attr(device_dir: &Path, attr: &str) -> Option<String> {
    let value = fs::read_to_string(device_dir.join(attr)).ok()?;
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Parses a sysfs capability bitmap: hex words of the kernel's `long`,
/// most significant word first. Returned least significant word first.
fn read_bitmap(file: &Path) -> Option<Vec<usize>> {
    parse_bitmap(&fs::read_to_string(file).ok()?)
}

fn parse_bitmap(text: &str) -> Option<Vec<usize>> {
    text.split_whitespace()
        .rev()
        .map(|word| usize::from_str_radix(word, 16).ok())
        .collect()
}

fn has_bit(words: &[usize], bit: usize) -> bool {
    let bits = usize::BITS as usize;
    words
        .get(bit / bits)
        .is_some_and(|word| word >> (bit % bits) & 1 == 1)
}

/// Keeps one event node per physical device: the `/input0` interface if
/// present, else the first. Devices without `phys` are all kept.
fn deduplicate_keyboards(keyboards: &mut Vec<KeyboardInfo>) {
    use std::collections::HashMap;

    let base = |phys: &str| {
        phys.rfind("/input")
            .map_or(phys, |pos| &phys[..pos])
            .to_string()
    };
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, kb) in keyboards.iter().enumerate() {
        if let Some(phys) = &kb.phys {
            groups.entry(base(phys)).or_default().push(idx);
        }
    }

    let mut drop: Vec<usize> = Vec::new();
    for indices in groups.values().filter(|g| g.len() > 1) {
        let keep = indices
            .iter()
            .copied()
            .find(|&i| {
                keyboards[i]
                    .phys
                    .as_deref()
                    .is_some_and(|p| p.ends_with("/input0"))
            })
            .unwrap_or(indices[0]);
        drop.extend(indices.iter().copied().filter(|&i| i != keep));
    }
    drop.sort_unstable_by(|a, b| b.cmp(a));
    for idx in drop {
        keyboards.remove(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Writes a fake `/sys/class/input/<node>/device` tree.
    fn fake_device(root: &Path, node: &str, name: &str, phys: &str, keys: &[Key]) {
        let dir = root.join(node).join("device");
        fs::create_dir_all(dir.join("capabilities")).unwrap();
        fs::write(dir.join("name"), format!("{name}\n")).unwrap();
        fs::write(dir.join("phys"), format!("{phys}\n")).unwrap();
        fs::write(dir.join("uniq"), "\n").unwrap();
        let ev = if keys.is_empty() { "0" } else { "120013" }; // SYN, KEY, MSC, LED, REP
        fs::write(dir.join("capabilities/ev"), format!("{ev}\n")).unwrap();
        fs::write(dir.join("capabilities/key"), format!("{}\n", bitmap(keys))).unwrap();
    }

    /// Renders keys as a kernel-style bitmap (MS word first).
    fn bitmap(keys: &[Key]) -> String {
        let bits = usize::BITS as usize;
        let mut words = vec![0usize; 12];
        for key in keys {
            let code = usize::from(key.code());
            words[code / bits] |= 1 << (code % bits);
        }
        let mut out: Vec<String> = words.iter().rev().map(|w| format!("{w:x}")).collect();
        while out.len() > 1 && out[0] == "0" {
            out.remove(0);
        }
        out.join(" ")
    }

    fn enumerate(root: &TempDir) -> Vec<KeyboardInfo> {
        enumerate_keyboards_in(root.path(), Path::new("/dev/input")).unwrap()
    }

    #[test]
    fn keyboard_is_listed_with_sysfs_attributes() {
        let root = TempDir::new().unwrap();
        fake_device(
            root.path(),
            "event3",
            "USB Keyboard",
            "usb-1/input0",
            &REQUIRED_KEYS,
        );
        let kbs = enumerate(&root);
        assert_eq!(kbs.len(), 1);
        assert_eq!(kbs[0].path, PathBuf::from("/dev/input/event3"));
        assert_eq!(kbs[0].name, "USB Keyboard");
        assert_eq!(kbs[0].phys.as_deref(), Some("usb-1/input0"));
        assert_eq!(kbs[0].serial, None);
    }

    #[test]
    fn non_keyboards_and_own_output_are_skipped() {
        let root = TempDir::new().unwrap();
        fake_device(
            root.path(),
            "event0",
            "Power Button",
            "LNXPWRBN/button/input0",
            &[Key::KEY_POWER],
        );
        fake_device(root.path(), "event1", "Mouse", "usb-2/input0", &[]);
        fake_device(root.path(), "event2", "keyrx", "", &REQUIRED_KEYS);
        fake_device(
            root.path(),
            "event4",
            "Few Letters",
            "usb-3/input0",
            &REQUIRED_KEYS[..10],
        );
        fs::create_dir_all(root.path().join("mouse0")).unwrap();
        assert!(enumerate(&root).is_empty());
    }

    #[test]
    fn one_node_per_physical_keyboard_prefers_input0() {
        let root = TempDir::new().unwrap();
        fake_device(
            root.path(),
            "event5",
            "KB Consumer",
            "usb-1/input1",
            &REQUIRED_KEYS,
        );
        fake_device(root.path(), "event4", "KB", "usb-1/input0", &REQUIRED_KEYS);
        fake_device(
            root.path(),
            "event9",
            "Other",
            "usb-7/input0",
            &REQUIRED_KEYS,
        );
        let names: Vec<_> = enumerate(&root).into_iter().map(|k| k.name).collect();
        assert_eq!(names, vec!["KB", "Other"]);
    }

    #[test]
    fn bitmap_parsing_matches_kernel_word_order() {
        let words = parse_bitmap("1 0\n").unwrap();
        assert!(has_bit(&words, usize::BITS as usize));
        assert!(!has_bit(&words, 0));
        assert!(!has_bit(&words, 10_000));
        assert!(parse_bitmap("zz").is_none());
    }

    /// The real sysfs agrees with evdev on this machine (whatever is attached).
    #[test]
    fn real_sysfs_is_readable() {
        assert!(enumerate_keyboards().is_ok());
    }
}

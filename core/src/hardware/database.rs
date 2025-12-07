use crate::hardware::{DeviceClass, HardwareInfo, HardwareProfile, ProfileSource, TimingConfig};
use std::collections::HashMap;

/// Stores hardware profiles keyed by vendor/product identifiers with class-based fallbacks.
#[derive(Debug, Clone)]
pub struct ProfileDatabase {
    builtin: HashMap<(u16, u16), HardwareProfile>,
}

impl ProfileDatabase {
    /// Build a database seeded with bundled builtin profiles.
    pub fn with_builtin() -> Self {
        Self::from_profiles(builtin_profiles())
    }

    /// Build a database from an arbitrary set of profiles (useful for tests or sync).
    pub fn from_profiles<I>(profiles: I) -> Self
    where
        I: IntoIterator<Item = HardwareProfile>,
    {
        let mut builtin = HashMap::new();
        for profile in profiles {
            builtin.insert((profile.vendor_id, profile.product_id), profile);
        }

        Self { builtin }
    }

    /// Iterate over builtin profiles for inspection or exporting.
    pub fn builtin_profiles(&self) -> impl Iterator<Item = &HardwareProfile> {
        self.builtin.values()
    }

    /// Lookup a profile by vendor and product identifiers.
    pub fn lookup(&self, vendor_id: u16, product_id: u16) -> Option<&HardwareProfile> {
        self.builtin.get(&(vendor_id, product_id))
    }

    /// Resolve the best profile for detected hardware, preferring VID/PID matches then class defaults.
    pub fn resolve(&self, hardware: &HardwareInfo) -> HardwareProfile {
        self.lookup(hardware.vendor_id, hardware.product_id)
            .cloned()
            .unwrap_or_else(|| self.class_default(hardware))
    }

    fn class_default(&self, hardware: &HardwareInfo) -> HardwareProfile {
        let timing = match hardware.device_class {
            DeviceClass::MechanicalKeyboard => TimingConfig {
                debounce_ms: 4,
                repeat_delay_ms: 235,
                repeat_rate_ms: 27,
                scan_interval_us: 850,
            },
            DeviceClass::MembraneKeyboard => TimingConfig {
                debounce_ms: 7,
                repeat_delay_ms: 270,
                repeat_rate_ms: 34,
                scan_interval_us: 1200,
            },
            DeviceClass::LaptopKeyboard => TimingConfig {
                debounce_ms: 5,
                repeat_delay_ms: 260,
                repeat_rate_ms: 31,
                scan_interval_us: 950,
            },
            DeviceClass::VirtualKeyboard => TimingConfig {
                debounce_ms: 2,
                repeat_delay_ms: 200,
                repeat_rate_ms: 25,
                scan_interval_us: 700,
            },
            DeviceClass::Unknown => TimingConfig::default(),
        };

        let name = match hardware.device_class {
            DeviceClass::MechanicalKeyboard => "Mechanical Baseline",
            DeviceClass::MembraneKeyboard => "Membrane Baseline",
            DeviceClass::LaptopKeyboard => "Laptop Baseline",
            DeviceClass::VirtualKeyboard => "Virtual Device Baseline",
            DeviceClass::Unknown => "Default Keyboard",
        };

        HardwareProfile::new(
            hardware.vendor_id,
            hardware.product_id,
            name,
            timing,
            ProfileSource::Builtin,
        )
    }
}

fn builtin_profiles() -> Vec<HardwareProfile> {
    vec![
        HardwareProfile::new(
            0x1b1c,
            0x1b2e,
            "Corsair K70 Pro",
            TimingConfig {
                debounce_ms: 4,
                repeat_delay_ms: 230,
                repeat_rate_ms: 26,
                scan_interval_us: 800,
            },
            ProfileSource::Builtin,
        ),
        HardwareProfile::new(
            0x3434,
            0x0055,
            "Glorious GMMK Pro",
            TimingConfig {
                debounce_ms: 3,
                repeat_delay_ms: 225,
                repeat_rate_ms: 25,
                scan_interval_us: 760,
            },
            ProfileSource::Builtin,
        ),
        HardwareProfile::new(
            0x17ef,
            0x60ee,
            "ThinkPad Compact USB Keyboard",
            TimingConfig {
                debounce_ms: 5,
                repeat_delay_ms: 255,
                repeat_rate_ms: 30,
                scan_interval_us: 900,
            },
            ProfileSource::Builtin,
        ),
        HardwareProfile::new(
            0x046d,
            0xc31c,
            "Logitech K120",
            TimingConfig {
                debounce_ms: 7,
                repeat_delay_ms: 270,
                repeat_rate_ms: 34,
                scan_interval_us: 1200,
            },
            ProfileSource::Builtin,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hardware(vendor_id: u16, product_id: u16, device_class: DeviceClass) -> HardwareInfo {
        HardwareInfo {
            vendor_id,
            product_id,
            vendor_name: None,
            product_name: None,
            device_class,
        }
    }

    #[test]
    fn lookup_returns_exact_profile() {
        let db = ProfileDatabase::with_builtin();
        let profile = db.lookup(0x1b1c, 0x1b2e).expect("profile present");
        assert_eq!(profile.name, "Corsair K70 Pro");
        assert_eq!(profile.source, ProfileSource::Builtin);
    }

    #[test]
    fn resolve_prefers_builtin_over_class_default() {
        let db = ProfileDatabase::with_builtin();
        let hw = hardware(0x3434, 0x0055, DeviceClass::MechanicalKeyboard);
        let profile = db.resolve(&hw);
        assert_eq!(profile.name, "Glorious GMMK Pro");
        assert_eq!(profile.vendor_id, hw.vendor_id);
        assert_eq!(profile.product_id, hw.product_id);
    }

    #[test]
    fn resolve_falls_back_to_class_default_when_missing() {
        let db = ProfileDatabase::with_builtin();
        let hw = hardware(0x9999, 0x0001, DeviceClass::MembraneKeyboard);
        let profile = db.resolve(&hw);
        assert_eq!(profile.vendor_id, 0x9999);
        assert_eq!(profile.product_id, 0x0001);
        assert_eq!(profile.name, "Membrane Baseline");
        assert_eq!(profile.source, ProfileSource::Builtin);
        assert!(profile.timing.debounce_ms >= 6);
    }
}

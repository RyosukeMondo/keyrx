//! Test device deduplication

use keyrx_daemon::device_manager;

fn main() {
    println!("Testing device enumeration with deduplication...\n");

    match device_manager::enumerate_keyboards() {
        Ok(devices) => {
            println!("Found {} device(s):\n", devices.len());
            for dev in devices {
                println!("Device: {}", dev.name);
                println!("  Path: {}", dev.path.display());
                println!("  Serial: {:?}", dev.serial);
                println!("  Physical: {:?}", dev.phys);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Error enumerating devices: {}", e);
        }
    }
}

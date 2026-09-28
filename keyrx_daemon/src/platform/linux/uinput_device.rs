//! Raw uinput device creation with every `EV_KEY` code and `EV_REL` axis
//! enabled.
//!
//! The `uinput` crate's `Builder::event(Keyboard::All)` only registers the
//! codes it has enum variants for (letters, digits, function keys, a
//! curated set of media keys, ...). It has no variant for vendor/consumer
//! keys such as `KEY_BRIGHTNESSUP` or `KEY_MICMUTE`, and no way to enable
//! `EV_REL` axes at all. A device created that way silently rejects (or
//! the caller must silently drop) any event outside that list.
//!
//! This module bypasses the crate's `Builder` and talks to `/dev/uinput`
//! directly to enable the *entire* `EV_KEY` (0..=KEY_MAX) and `EV_REL`
//! (0..=REL_MAX) ranges once, at creation. That is simpler and more robust
//! than mirroring each grabbed device's own capabilities: the daemon's
//! output device is one process-wide virtual keyboard shared by every
//! captured input device, so per-device capabilities would otherwise have
//! to be reconciled (and the device probably recreated - uinput cannot add
//! capabilities after `UI_DEV_CREATE`) on every hotplug or reload.
//!
//! The resulting fd is handed to `uinput::device::Device` (its `Device::new`
//! and `Device::write` are public, just `#[doc(hidden)]`), so callers keep
//! using the same `Device` type as before for `.press()`/`.release()`/
//! `.synchronize()`.

use std::ffi::CString;
use std::io;
use std::os::fd::{BorrowedFd, RawFd};

use nix::fcntl::{self, OFlag};
use nix::sys::stat::Mode;
use nix::unistd;

use uinput::device::Device as UInputDevice;

/// Creates a uinput virtual keyboard named `name` with every `EV_KEY` code
/// and `EV_REL` axis enabled (see module docs).
pub(crate) fn create_full_capability_device(name: &str) -> io::Result<UInputDevice> {
    let fd = fcntl::open(
        "/dev/uinput",
        OFlag::O_WRONLY | OFlag::O_NONBLOCK,
        Mode::empty(),
    )
    .map_err(io::Error::from)?;

    if let Err(e) = enable_capabilities(fd).and_then(|()| write_metadata(fd, name)) {
        let _ = unistd::close(fd);
        return Err(e);
    }

    // SAFETY: `fd` was just opened and configured above; `ui_dev_create`
    // is the standard uinput finalization ioctl.
    let created = unsafe { uinput_sys::ui_dev_create(fd) };
    if created < 0 {
        let err = io::Error::last_os_error();
        let _ = unistd::close(fd);
        return Err(err);
    }

    // `Device` takes ownership: `UI_DEV_DESTROY` runs on drop (same as the
    // devices `uinput::Builder::create()` produces).
    Ok(UInputDevice::new(fd))
}

/// Sets every `EV_KEY` and `EV_REL` capability bit, in the same order the
/// kernel expects: `UI_SET_EVBIT` for the event type before any
/// `UI_SET_KEYBIT`/`UI_SET_RELBIT` calls for its codes.
fn enable_capabilities(fd: RawFd) -> io::Result<()> {
    set_evbit(fd, uinput_sys::EV_KEY)?;
    for code in 0..=uinput_sys::KEY_MAX {
        set_bit(fd, uinput_sys::ui_set_keybit, code)?;
    }
    set_evbit(fd, uinput_sys::EV_REL)?;
    for axis in 0..=uinput_sys::REL_MAX {
        set_bit(fd, uinput_sys::ui_set_relbit, axis)?;
    }
    Ok(())
}

fn set_evbit(fd: RawFd, event_type: i32) -> io::Result<()> {
    // SAFETY: `fd` is a valid, open uinput fd owned by the caller.
    let ret = unsafe { uinput_sys::ui_set_evbit(fd, event_type) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn set_bit(fd: RawFd, ioctl: unsafe fn(RawFd, i32) -> i32, value: i32) -> io::Result<()> {
    // SAFETY: `fd` is a valid, open uinput fd owned by the caller.
    let ret = unsafe { ioctl(fd, value) };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Writes the `uinput_user_dev` struct (device name; bus/vendor/product
/// left at 0, absolute-axis tables unused) that `UI_DEV_CREATE` reads.
fn write_metadata(fd: RawFd, name: &str) -> io::Result<()> {
    let cname = CString::new(name).map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let bytes = cname.as_bytes_with_nul();
    if bytes.len() > uinput_sys::UINPUT_MAX_NAME_SIZE as usize {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("device name '{name}' too long for uinput"),
        ));
    }

    // SAFETY: `uinput_user_dev` is a C struct of plain integers/arrays;
    // all-zeroes is a valid value for it.
    let mut dev: uinput_sys::uinput_user_dev = unsafe { std::mem::zeroed() };
    for (dst, &src) in dev.name.iter_mut().zip(bytes) {
        *dst = src as std::os::raw::c_char;
    }

    // SAFETY: `dev` is a `#[repr(C)]` plain-old-data struct; reading it as
    // a byte slice of its exact size is what `UI_DEV_CREATE` expects on
    // the other end (a `write(2)` of the struct).
    let dev_bytes = unsafe {
        std::slice::from_raw_parts(
            (&dev as *const uinput_sys::uinput_user_dev).cast::<u8>(),
            std::mem::size_of::<uinput_sys::uinput_user_dev>(),
        )
    };
    // SAFETY: `fd` is a valid, open uinput fd owned by the caller for the
    // duration of this call.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    unistd::write(borrowed, dev_bytes).map_err(io::Error::from)?;
    Ok(())
}

/// Writes a raw `(type, code, value)` triple followed by a `SYN_REPORT`,
/// bypassing the `KeyCode`-typed `press`/`release`/`position` helpers.
/// Used to forward events this daemon cannot represent as a `KeyCode`
/// (H6): unmapped `EV_KEY` codes and non-key event types.
pub(crate) fn write_raw(
    device: &mut UInputDevice,
    event_type: u16,
    code: u16,
    value: i32,
) -> uinput::Result<()> {
    device.write(i32::from(event_type), i32::from(code), value)?;
    device.synchronize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_full_capability_device_succeeds_and_enables_full_range() {
        crate::skip_if_no_uinput!();
        let device = create_full_capability_device("keyrx-uinput-device-test")
            .expect("full-capability device creation should succeed with uinput access");
        drop(device);
    }
}

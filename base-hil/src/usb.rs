//! Enumerate USB opt-in — feature `hil_usb` (rusb). Default build: sempre `false`.

/// Retorna true se existir dispositivo USB com VID:PID.
/// Sem feature `hil_usb`: sempre false (CI default sem libusb).
#[cfg(feature = "hil_usb")]
pub(crate) fn usb_device_present(vid: u16, pid: u16) -> bool {
    match rusb::devices() {
        Ok(list) => {
            let found = list.iter().any(|dev| {
                dev.device_descriptor()
                    .map(|d| d.vendor_id() == vid && d.product_id() == pid)
                    .unwrap_or(false)
            });
            if found {
                tracing::info!(
                    "[HIL][EXPERIMENTAL] USB device {:04x}:{:04x} present (hil_usb)",
                    vid,
                    pid
                );
            } else {
                tracing::debug!(
                    "[HIL][EXPERIMENTAL] USB scan: {:04x}:{:04x} not found",
                    vid,
                    pid
                );
            }
            found
        }
        Err(e) => {
            tracing::warn!(
                "[HIL][EXPERIMENTAL] USB enumerate failed ({e}) — treating as Simulated"
            );
            false
        }
    }
}

#[cfg(not(feature = "hil_usb"))]
pub(crate) fn usb_device_present(_vid: u16, _pid: u16) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn without_device_returns_bool() {
        // Sem hardware: false. Com feature + device: pode ser true — só garante que não panic.
        let _ = usb_device_present(0xCAFE, 0x4007);
    }

    #[cfg(not(feature = "hil_usb"))]
    #[test]
    fn default_build_never_claims_usb() {
        assert!(!usb_device_present(0xCAFE, 0x4007));
        assert!(!usb_device_present(0x0000, 0x0000));
    }
}

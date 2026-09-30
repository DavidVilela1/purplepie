//! Capture of asynchronous GPU failures (ADR-017).
//!
//! wgpu reports validation errors, out-of-memory and internal errors through
//! an "uncaptured error" callback, and a lost device through a separate
//! callback. By default wgpu **panics** on uncaptured errors. PurplePie
//! installs its own callbacks that record the first fault in a [`FaultSlot`].
//! The renderer then turns it into [`crate::Error::Render`], so the engine
//! exits cleanly with a readable error chain.

use std::sync::{Arc, Mutex, PoisonError};

/// A fatal GPU condition. Crate-private: it reaches games only as the boxed
/// `source` of [`crate::Error::Render`].
#[derive(Debug, thiserror::Error)]
pub(crate) enum GpuFault {
    /// wgpu raised a validation, out-of-memory or internal error that no error
    /// scope caught. Validation errors are PurplePie bugs.
    #[error("uncaptured wgpu error")]
    Uncaptured(#[source] wgpu::Error),

    /// The GPU device stopped working (driver crash or reset, GPU removed).
    #[error("GPU device lost: {0}")]
    DeviceLost(String),

    /// The window's surface was lost (`CurrentSurfaceTexture::Lost`), typically
    /// because the window or display connection went away. Not recovered:
    /// recreating a surface for a destroyed window panics inside wgpu-hal 30.0.1
    /// (measured, ADR-017).
    #[error("the window's GPU surface was lost")]
    SurfaceLost,

    /// `get_current_texture` reported a validation failure, but no uncaptured
    /// error was recorded to explain it.
    #[error("the surface texture could not be acquired (validation failure)")]
    AcquireValidation,
}

/// Shared slot holding the first [`GpuFault`] reported by wgpu's callbacks.
///
/// `Arc<Mutex<..>>` is required here: wgpu's callbacks must be
/// `Send + Sync + 'static` and may run on another thread.
#[derive(Debug, Clone, Default)]
pub(crate) struct FaultSlot(Arc<Mutex<Option<GpuFault>>>);

impl FaultSlot {
    /// Installs PurplePie's callbacks on `device`, replacing wgpu's panicking default.
    pub(crate) fn install(&self, device: &wgpu::Device) {
        let slot = self.clone();
        device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            log::error!("wgpu error: {error}");
            slot.record(GpuFault::Uncaptured(error));
        }));

        let slot = self.clone();
        device.set_device_lost_callback(move |reason, message| {
            if let Some(fault) = device_lost_fault(reason, message) {
                log::error!("{fault}");
                slot.record(fault);
            }
        });
    }

    /// Stores `fault` unless an earlier one is already waiting (the first fault is the root cause).
    pub(crate) fn record(&self, fault: GpuFault) {
        let mut slot = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.is_none() {
            *slot = Some(fault);
        }
    }

    /// Removes and returns the recorded fault, if any.
    pub(crate) fn take(&self) -> Option<GpuFault> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).take()
    }
}

/// Decides whether a device-lost notification is a fault.
///
/// `Destroyed` only happens when PurplePie calls `Device::destroy` itself, which
/// is intentional. `Unknown` is a real loss (driver reset, GPU removed).
fn device_lost_fault(reason: wgpu::DeviceLostReason, message: String) -> Option<GpuFault> {
    match reason {
        wgpu::DeviceLostReason::Destroyed => None,
        wgpu::DeviceLostReason::Unknown => Some(GpuFault::DeviceLost(if message.is_empty() {
            "no details reported".to_owned()
        } else {
            message
        })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_fault_wins_and_take_clears() {
        let slot = FaultSlot::default();
        assert!(slot.take().is_none());
        slot.record(GpuFault::DeviceLost("first".into()));
        slot.record(GpuFault::AcquireValidation);
        match slot.take() {
            Some(GpuFault::DeviceLost(message)) => assert_eq!(message, "first"),
            other => panic!("expected the first fault, got {other:?}"),
        }
        assert!(slot.take().is_none());
    }

    #[test]
    fn clones_share_the_same_slot() {
        let slot = FaultSlot::default();
        slot.clone().record(GpuFault::AcquireValidation);
        assert!(matches!(slot.take(), Some(GpuFault::AcquireValidation)));
    }

    #[test]
    fn destroyed_device_is_not_a_fault() {
        assert!(device_lost_fault(wgpu::DeviceLostReason::Destroyed, String::new()).is_none());
    }

    #[test]
    fn unknown_device_loss_is_a_fault_with_a_message() {
        let fault = device_lost_fault(wgpu::DeviceLostReason::Unknown, "driver reset".into());
        assert_eq!(
            fault.map(|f| f.to_string()).as_deref(),
            Some("GPU device lost: driver reset")
        );
        let fault = device_lost_fault(wgpu::DeviceLostReason::Unknown, String::new());
        assert_eq!(
            fault.map(|f| f.to_string()).as_deref(),
            Some("GPU device lost: no details reported")
        );
    }

    /// Exercises the real wgpu callbacks on a headless device. Needs a GPU
    /// adapter (hardware or software, e.g. Mesa lavapipe), which CI runners do
    /// not have. Run with `cargo test -- --ignored`.
    #[test]
    #[ignore = "needs a GPU adapter; run with `cargo test -- --ignored`"]
    fn installed_callbacks_capture_validation_errors_instead_of_panicking() {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .expect("a GPU adapter is required for this ignored test");
        let (device, _queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
                .expect("device");
        let slot = FaultSlot::default();
        slot.install(&device);

        // MAP_READ and MAP_WRITE together are invalid, so this raises a validation error.
        let _invalid = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("deliberately invalid"),
            size: 16,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::MAP_WRITE,
            mapped_at_creation: false,
        });

        match slot.take() {
            Some(GpuFault::Uncaptured(wgpu::Error::Validation { .. })) => {}
            other => panic!("expected a captured validation error, got {other:?}"),
        }

        // An intentional destroy is not reported as a fault.
        device.destroy();
        let _ = device.poll(wgpu::PollType::wait_indefinitely());
        assert!(slot.take().is_none());
    }
}

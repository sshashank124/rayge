use ash::khr;

use super::instance::Instance;

pub mod instance {
    use ash::{ext, khr, vk};

    pub const REQUIRED: &[*const std::ffi::c_char] = &[
        khr::portability_enumeration::NAME.as_ptr(),
        // Surface
        ext::metal_surface::NAME.as_ptr(),
        khr::surface::NAME.as_ptr(),
    ];

    pub const FLAGS: vk::InstanceCreateFlags = vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
}

pub mod device {
    use core::ffi;
    use std::collections::HashSet;

    use ash::{khr, vk};

    use super::super::Instance;

    type Result<T> = core::result::Result<T, DeviceExtensionsError>;

    // TODO: split into required and optional extensions
    pub const REQUIRED: &[&ffi::CStr] = &[
        // Core
        khr::swapchain::NAME,
        // Acceleration Structure
        // khr::acceleration_structure::NAME,
        // khr::deferred_host_operations::NAME,
        // Ray Tracing
        // khr::ray_tracing_pipeline::NAME,
        // Additional
        // ext::memory_priority::NAME,
        // ext::pageable_device_local_memory::NAME,
    ];

    pub fn supported_by(instance: &Instance, physical_device: vk::PhysicalDevice) -> Result<bool> {
        let available = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .map_err(DeviceExtensionsError::Enumerate)?
        };

        let available: HashSet<_> = available
            .iter()
            .map(vk::ExtensionProperties::extension_name_as_c_str)
            .collect::<core::result::Result<_, _>>()
            .map_err(DeviceExtensionsError::Parse)?;

        Ok(REQUIRED.iter().all(|required| available.contains(required)))
    }

    #[derive(Debug, thiserror::Error)]
    pub enum DeviceExtensionsError {
        #[error("failed to enumerate device extensions / {0}")]
        Enumerate(vk::Result),
        #[error("failed to parse device extension names / {0}")]
        Parse(#[from] ffi::FromBytesUntilNulError),
    }
}

pub struct Handles {
    swapchain: khr::swapchain::Device,
}

impl Handles {
    pub fn new(instance: &Instance, device: &ash::Device) -> Self {
        let swapchain = khr::swapchain::Device::new(instance, device);
        Self { swapchain }
    }
}

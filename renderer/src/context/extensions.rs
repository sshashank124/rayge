use ash::{ext, khr};

pub mod instance {
    use ash::{ext, khr};

    pub const REQUIRED: &[*const std::ffi::c_char] = &[
        ext::debug_utils::NAME.as_ptr(),
        // Surface
        khr::surface::NAME.as_ptr(),
        khr::win32_surface::NAME.as_ptr(),
    ];
}

pub mod device {
    use std::collections::HashSet;

    use ash::{ext, khr, vk};

    use super::super::instance;

    type Result<T> = core::result::Result<T, Error>;

    // TODO: split into required and optional extensions
    pub const REQUIRED: &[&std::ffi::CStr] = &[
        // Surface
        khr::swapchain::NAME,
        // Acceleration Structure
        khr::acceleration_structure::NAME,
        khr::deferred_host_operations::NAME,
        // Ray Tracing
        khr::ray_tracing_pipeline::NAME,
        // Additional
        ext::memory_priority::NAME,
        ext::pageable_device_local_memory::NAME,
    ];

    pub fn supported_by(
        instance: &instance::Instance,
        physical_device: vk::PhysicalDevice,
    ) -> Result<bool> {
        let available = unsafe {
            instance
                .enumerate_device_extension_properties(physical_device)
                .map_err(Error::Enumerate)?
        };

        let available: HashSet<_> = available
            .iter()
            .map(vk::ExtensionProperties::extension_name_as_c_str)
            .collect::<core::result::Result<_, _>>()
            .map_err(Error::Parse)?;

        Ok(REQUIRED.iter().all(|required| available.contains(required)))
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("failed to enumerate device extensions / {0}")]
        Enumerate(vk::Result),
        #[error("failed to parse device extension names / {0}")]
        Parse(#[from] std::ffi::FromBytesUntilNulError),
    }
}

pub struct Handles {
    pub debug_utils: ext::debug_utils::Device,
    pub swapchain: khr::swapchain::Device,
}

impl Handles {
    pub fn new(instance: &super::instance::Instance, device: &ash::Device) -> Self {
        let debug_utils = ext::debug_utils::Device::new(instance, device);
        let swapchain = khr::swapchain::Device::new(instance, device);
        Self {
            debug_utils,
            swapchain,
        }
    }
}

use ash::vk;

use super::{extensions, features, instance::Instance, properties::Properties, surface};

type Result<T> = core::result::Result<T, PhysicalDeviceError>;

pub struct PhysicalDevice {
    properties: Properties,
    handle: vk::PhysicalDevice,
}

impl PhysicalDevice {
    pub fn new(instance: &Instance, surface: &surface::Handle) -> Result<(Self, surface::Config)> {
        let possible_physical_devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(PhysicalDeviceError::Enumerate)?;

        for physical_device in possible_physical_devices {
            if let Some(result) = Self::try_create(instance, physical_device, surface)? {
                return Ok(result);
            }
        }

        Err(PhysicalDeviceError::NoSuitableCandidate)
    }

    fn try_create(
        instance: &Instance,
        handle: vk::PhysicalDevice,
        surface: &surface::Handle,
    ) -> Result<Option<(Self, surface::Config)>> {
        Ok(
            if extensions::device::supported_by(instance, handle)?
                && features::supported_by(instance, handle)
                && let Some(surface_config) = surface.get_config(handle)?
            {
                let physical_device = Self {
                    properties: Properties::get_supported(instance, handle),
                    handle,
                };
                Some((physical_device, surface_config))
            } else {
                None
            },
        )
    }
}

impl std::ops::Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl core::fmt::Debug for PhysicalDevice {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PhysicalDevice")
            .field("properties", &self.properties)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PhysicalDeviceError {
    #[error("failed to enumerate physical devices / {0}")]
    Enumerate(vk::Result),
    #[error("device extensions / {0}")]
    DeviceExtensions(#[from] extensions::device::DeviceExtensionsError),
    #[error("surface / {0}")]
    Surface(#[from] surface::SurfaceError),
    #[error("no suitable physical device found")]
    NoSuitableCandidate,
}

use std::ops::Deref;

use ash::vk;

use super::{extensions, features, instance::Instance, properties::Properties, surface};

type Result<T> = core::result::Result<T, PhysicalDeviceError>;

pub struct PhysicalDevice {
    physical_device: vk::PhysicalDevice,
    properties: Properties,
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
        physical_device: vk::PhysicalDevice,
        surface: &surface::Handle,
    ) -> Result<Option<(Self, surface::Config)>> {
        Ok(
            if extensions::device::supported_by(instance, physical_device)?
                && features::supported_by(instance, physical_device)
                && let Some(surface_config) = surface.get_config(physical_device)?
            {
                let physical_device = Self {
                    physical_device,
                    properties: Properties::get_supported(instance, physical_device),
                };
                Some((physical_device, surface_config))
            } else {
                None
            },
        )
    }
}

impl Deref for PhysicalDevice {
    type Target = vk::PhysicalDevice;
    fn deref(&self) -> &Self::Target {
        &self.physical_device
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

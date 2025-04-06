pub mod device;
mod extensions;
mod features;
mod instance;
mod physical_device;
mod properties;
mod queue;
pub mod surface;

use raw_window_handle::HasWindowHandle;

type Result<T> = core::result::Result<T, Error>;

pub struct Context {
    device: device::Device,
    pub surface: surface::Surface,
    physical_device: physical_device::PhysicalDevice,
    _instance: instance::Instance,
}

impl Context {
    pub(super) fn new(window: &impl HasWindowHandle) -> Result<Self> {
        let instance = instance::Instance::new()?;

        let surface_handle = surface::Handle::new(&instance, window)?;

        let (physical_device, surface_config) =
            physical_device::PhysicalDevice::new(&instance, &surface_handle)?;

        let surface = surface::Surface::new(surface_config, surface_handle);

        let device = device::Device::new(&instance, &physical_device, &surface)?;

        let context = Self {
            device,
            surface,
            physical_device,
            _instance: instance,
        };

        tracing::debug!("Context initialized: {context:?}");

        Ok(context)
    }

    pub fn refresh_surface_capabilities(&mut self) -> Result<bool> {
        Ok(self.surface.refresh_capabilities(&self.physical_device)?)
    }
}

impl std::ops::Deref for Context {
    type Target = device::Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl core::fmt::Debug for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            device,
            surface,
            physical_device,
            _instance: _,
        } = self;
        f.debug_struct("Context")
            .field("device", device)
            .field("surface", surface)
            .field("physical_device", physical_device)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("instance / {0}")]
    Instance(#[from] instance::Error),
    #[error("physical device / {0}")]
    PhysicalDevice(#[from] physical_device::Error),
    #[error("surface / {0}")]
    Surface(#[from] surface::Error),
    #[error("device / {0}")]
    Device(#[from] device::Error),
}

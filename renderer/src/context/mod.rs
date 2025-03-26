mod device;
mod extensions;
mod features;
mod instance;
mod physical_device;
mod properties;
mod queue;
mod surface;

use raw_window_handle::HasWindowHandle;

use device::Device;
use instance::Instance;
use physical_device::PhysicalDevice;
use surface::Surface;

type Result<T> = core::result::Result<T, ContextError>;

pub struct Context {
    device: Device,
    pub surface: Surface,
    physical_device: PhysicalDevice,
    instance: Instance,
}

impl Context {
    pub(super) fn new(window: &impl HasWindowHandle) -> Result<Self> {
        let instance = Instance::new()?;

        let surface_handle = surface::Handle::new(&instance, window)?;

        let (physical_device, surface_config) = PhysicalDevice::new(&instance, &surface_handle)?;

        let surface = Surface::new(surface_config, surface_handle);

        let device = Device::new(&instance, &physical_device, &surface)?;

        let context = Self {
            device,
            surface,
            physical_device,
            instance,
        };

        tracing::debug!("Context initialized: {context:?}");

        Ok(context)
    }
}

impl std::ops::Deref for Context {
    type Target = Device;
    fn deref(&self) -> &Self::Target {
        &self.device
    }
}

impl core::fmt::Debug for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Context")
            .field("device", &self.device)
            .field("surface", &self.surface)
            .field("physical_device", &self.physical_device)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ContextError {
    #[error("instance / {0}")]
    Instance(#[from] instance::InstanceError),
    #[error("physical device / {0}")]
    PhysicalDevice(#[from] physical_device::PhysicalDeviceError),
    #[error("surface / {0}")]
    Surface(#[from] surface::SurfaceError),
    #[error("device / {0}")]
    Device(#[from] device::DeviceError),
}

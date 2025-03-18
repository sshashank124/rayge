mod device;
mod extensions;
mod features;
mod instance;
mod physical_device;
mod properties;
mod queue;
mod surface;

use device::Device;
use physical_device::PhysicalDevice;
use raw_window_handle::HasWindowHandle;

use instance::Instance;
use surface::Surface;

type Result<T> = core::result::Result<T, ContextError>;

pub struct Context {
    device: Device,
    surface: Surface,
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

        Ok(Self {
            device,
            surface,
            physical_device,
            instance,
        })
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

use ash::{khr, vk};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use super::{instance::Instance, physical_device::PhysicalDevice};

pub mod conf {
    use ash::vk;

    pub const FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    pub const PREFERRED_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO_RELAXED;
    pub const FALLBACK_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO;
    pub const PREFERRED_IMAGE_COUNT: u32 = 3;
}

type Result<T> = core::result::Result<T, Error>;

pub struct Surface {
    pub config: Config,
    handle: Handle,
}

#[derive(Debug)]
pub struct Config {
    pub present_mode: vk::PresentModeKHR,
    pub extent: vk::Extent2D,
    pub image_count: u32,
}

pub struct Handle {
    surface: vk::SurfaceKHR,
    loader: khr::surface::Instance,
}

impl Surface {
    pub const fn new(config: Config, handle: Handle) -> Self {
        Self { config, handle }
    }

    pub fn refresh_capabilities(&mut self, physical_device: &PhysicalDevice) -> Result<bool> {
        Ok(self
            .config
            .update_with(&self.get_capabilities(**physical_device)?))
    }
}

impl Config {
    fn update_with(&mut self, capabilities: &vk::SurfaceCapabilitiesKHR) -> bool {
        self.extent = Handle::choose_extent(capabilities);
        self.extent.width != 0 && self.extent.height != 0
    }
}

impl Handle {
    pub fn new(instance: &Instance, window: &impl HasWindowHandle) -> Result<Self> {
        let surface = create_surface(instance, window)?;
        let loader = khr::surface::Instance::new(&instance.entry, instance);
        Ok(Self { surface, loader })
    }

    pub fn get_config(&self, physical_device: vk::PhysicalDevice) -> Result<Option<Config>> {
        let capabilities = self.get_capabilities(physical_device)?;

        let surface_formats = unsafe {
            self.loader
                .get_physical_device_surface_formats(physical_device, self.surface)
                .map_err(Error::GetConfigOptions)?
        };

        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
                .map_err(Error::GetConfigOptions)?
        };

        Ok(Self::choose_best_surface_format(&surface_formats).map(|_| {
            let extent = Self::choose_extent(&capabilities);
            let image_count = Self::choose_image_count(&capabilities);
            let present_mode = Self::choose_best_present_mode(&present_modes);

            Config {
                present_mode,
                extent,
                image_count,
            }
        }))
    }

    fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
                .map_err(Error::GetConfigOptions)
        }
    }

    fn choose_best_surface_format(
        formats: &[vk::SurfaceFormatKHR],
    ) -> Option<vk::SurfaceFormatKHR> {
        formats.iter().copied().find(|f| *f == conf::FORMAT)
    }

    fn choose_best_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        if present_modes.contains(&conf::PREFERRED_PRESENT_MODE) {
            conf::PREFERRED_PRESENT_MODE
        } else {
            conf::FALLBACK_PRESENT_MODE
        }
    }

    fn choose_extent(capabilities: &vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            return capabilities.current_extent;
        }

        vk::Extent2D {
            width: capabilities.current_extent.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: capabilities.current_extent.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }

    fn choose_image_count(
        vk::SurfaceCapabilitiesKHR {
            min_image_count,
            max_image_count,
            ..
        }: &vk::SurfaceCapabilitiesKHR,
    ) -> u32 {
        if *max_image_count < *min_image_count {
            conf::PREFERRED_IMAGE_COUNT.max(*min_image_count)
        } else {
            conf::PREFERRED_IMAGE_COUNT.clamp(*min_image_count, *max_image_count)
        }
    }

    pub fn is_supported_by(
        &self,
        physical_device: &PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        unsafe {
            self.loader
                .get_physical_device_surface_support(
                    **physical_device,
                    queue_family_index,
                    self.surface,
                )
                .map_err(Error::GetConfigOptions)
        }
    }
}

fn create_surface(instance: &Instance, window: &impl HasWindowHandle) -> Result<vk::SurfaceKHR> {
    match window.window_handle()?.as_raw() {
        RawWindowHandle::Win32(handle) => {
            let create_info = vk::Win32SurfaceCreateInfoKHR::default()
                .hwnd(handle.hwnd.get() as _)
                .hinstance(handle.hinstance.expect("No Win32 HINSTANCE found").get() as _);
            unsafe {
                khr::win32_surface::Instance::new(&instance.entry, instance)
                    .create_win32_surface(&create_info, None)
                    .map_err(Error::Create)
            }
        }
        _ => Err(Error::UnsupportedPlatform),
    }
}

impl std::ops::Deref for Surface {
    type Target = Handle;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl std::ops::Deref for Handle {
    type Target = vk::SurfaceKHR;
    fn deref(&self) -> &Self::Target {
        &self.surface
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        let Self { surface, loader } = self;
        unsafe {
            loader.destroy_surface(*surface, None);
        }
    }
}

impl core::fmt::Debug for Surface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { config, handle: _ } = self;
        f.debug_struct("Surface")
            .field("config", config)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to get window handle / {0}")]
    GetHandle(#[from] raw_window_handle::HandleError),
    #[error("failed to create window surface / {0}")]
    Create(vk::Result),
    #[error("unsupported windowing platform")]
    UnsupportedPlatform,
    #[error("unable to get config options / {0}")]
    GetConfigOptions(vk::Result),
}

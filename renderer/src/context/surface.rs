use ash::{ext, khr, vk};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use raw_window_metal::Layer;

use super::{instance::Instance, physical_device::PhysicalDevice};

mod conf {
    use ash::vk;

    pub const PREFERRED_SURFACE_FORMAT: vk::SurfaceFormatKHR = vk::SurfaceFormatKHR {
        format: vk::Format::B8G8R8A8_SRGB,
        color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
    };
    pub const PREFERRED_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO_RELAXED;
    pub const FALLBACK_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO;
}

type Result<T> = core::result::Result<T, SurfaceError>;

pub struct Surface {
    pub config: Config,
    handle: Handle,
}

#[derive(Debug)]
pub struct Config {
    pub surface_format: vk::SurfaceFormatKHR,
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
                .map_err(SurfaceError::GetConfigOptions)?
        };

        let present_modes = unsafe {
            self.loader
                .get_physical_device_surface_present_modes(physical_device, self.surface)
                .map_err(SurfaceError::GetConfigOptions)?
        };

        Ok(
            if !surface_formats.is_empty() && !present_modes.is_empty() {
                let surface_format = Self::choose_best_surface_format(&surface_formats);
                let extent = Self::choose_extent(&capabilities);
                let image_count = Self::choose_image_count(&capabilities);
                let present_mode = Self::choose_best_present_mode(&present_modes);

                Some(Config {
                    surface_format,
                    present_mode,
                    extent,
                    image_count,
                })
            } else {
                None
            },
        )
    }

    fn get_capabilities(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(physical_device, self.surface)
                .map_err(SurfaceError::GetConfigOptions)
        }
    }

    fn choose_best_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        if formats.contains(&conf::PREFERRED_SURFACE_FORMAT) {
            conf::PREFERRED_SURFACE_FORMAT
        } else {
            formats[0]
        }
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

    fn choose_image_count(capabilities: &vk::SurfaceCapabilitiesKHR) -> u32 {
        let image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count > 0 {
            image_count.min(capabilities.max_image_count)
        } else {
            image_count
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
                .map_err(SurfaceError::GetConfigOptions)
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
                    .map_err(SurfaceError::Create)
            }
        }
        RawWindowHandle::AppKit(handle) => {
            let layer = unsafe { Layer::from_ns_view(handle.ns_view) };
            let create_info =
                vk::MetalSurfaceCreateInfoEXT::default().layer(layer.as_ptr().as_ptr());
            unsafe {
                ext::metal_surface::Instance::new(&instance.entry, instance)
                    .create_metal_surface(&create_info, None)
                    .map_err(SurfaceError::Create)
            }
        }
        _ => Err(SurfaceError::UnsupportedPlatform),
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
        unsafe {
            self.loader.destroy_surface(self.surface, None);
        }
    }
}

impl core::fmt::Debug for Surface {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Surface")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SurfaceError {
    #[error("failed to get window handle / {0}")]
    GetHandle(#[from] raw_window_handle::HandleError),
    #[error("failed to create window surface / {0}")]
    Create(vk::Result),
    #[error("unsupported windowing platform")]
    UnsupportedPlatform,
    #[error("unable to get config options / {0}")]
    GetConfigOptions(vk::Result),
}

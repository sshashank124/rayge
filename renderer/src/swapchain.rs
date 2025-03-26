use ash::vk;

use crate::Ctx;

type Result<T> = core::result::Result<T, SwapchainError>;

pub struct Swapchain {
    pub handle: vk::SwapchainKHR,
    ctx: Ctx,
}

impl Swapchain {
    pub fn new(ctx: &Ctx) -> Result<Self> {
        let handle = {
            let create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(**ctx.surface)
                .min_image_count(ctx.surface.config.image_count)
                .image_format(ctx.surface.config.surface_format.format)
                .image_color_space(ctx.surface.config.surface_format.color_space)
                .image_extent(ctx.surface.config.extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(ctx.surface.config.present_mode)
                .clipped(true);

            unsafe {
                ctx.ext
                    .swapchain
                    .create_swapchain(&create_info, None)
                    .map_err(SwapchainError::Create)?
            }
        };

        Ok(Self {
            handle,
            ctx: ctx.clone(),
        })
    }
}

impl Drop for Swapchain {
    fn drop(&mut self) {
        unsafe { self.ctx.ext.swapchain.destroy_swapchain(self.handle, None) };
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SwapchainError {
    #[error("failed to create swapchain / {0}")]
    Create(vk::Result),
}

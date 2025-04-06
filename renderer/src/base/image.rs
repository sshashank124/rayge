use std::marker::ConstParamTy;

use ash::vk;

use crate::{
    context::{Context, device, surface},
    destroy::Destroy,
};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(ConstParamTy, Eq, PartialEq)]
pub enum Format {
    Hdr,
    Swapchain,
}

impl From<Format> for vk::Format {
    fn from(format: Format) -> Self {
        match format {
            Format::Hdr => Self::R32G32B32A32_SFLOAT,
            Format::Swapchain => surface::conf::FORMAT.format,
        }
    }
}

pub struct Image<const FORMAT: Format> {
    handle: vk::Image,
    view: vk::ImageView,
    extent: vk::Extent2D,
}

impl<const FORMAT: Format> Image<FORMAT> {
    pub fn new(ctx: &Context, handle: vk::Image, extent: vk::Extent2D, name: &str) -> Result<Self> {
        let view = {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(handle)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(FORMAT.into())
                .subresource_range(Self::subresource_range());

            unsafe {
                ctx.create_image_view(&create_info, None)
                    .map_err(Error::CreateView)?
            }
        };
        ctx.set_debug_name(view, &format!("{name}_image_view"))?;

        Ok(Self {
            handle,
            view,
            extent,
        })
    }

    const fn subresource_range() -> vk::ImageSubresourceRange {
        vk::ImageSubresourceRange {
            aspect_mask: Self::aspect_flags(),
            base_mip_level: 0,
            level_count: vk::REMAINING_MIP_LEVELS,
            base_array_layer: 0,
            layer_count: vk::REMAINING_ARRAY_LAYERS,
        }
    }

    const fn aspect_flags() -> vk::ImageAspectFlags {
        match FORMAT {
            Format::Hdr | Format::Swapchain => vk::ImageAspectFlags::COLOR,
        }
    }
}

impl<const FORMAT: Format> Destroy<Context> for Image<FORMAT> {
    fn destroy_with(&mut self, ctx: &Context) {
        let Self {
            handle: _,
            view,
            extent: _,
        } = self;
        unsafe { ctx.destroy_image_view(*view, None) };
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create image view / {0}")]
    CreateView(vk::Result),
    #[error("device / {0}")]
    Device(#[from] device::Error),
}

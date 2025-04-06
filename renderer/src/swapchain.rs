use ash::vk;

use crate::{
    base::image,
    context::{Context, surface},
    destroy::Destroy,
};

type Result<T> = core::result::Result<T, Error>;

mod conf {
    pub const BUFFERING: usize = 2;
}

pub struct Swapchain {
    syncs: [sync_state::SyncState; conf::BUFFERING],
    frame_idx: usize,
    images: Vec<image::Image<{ image::Format::Swapchain }>>,
    handle: vk::SwapchainKHR,
}

impl Swapchain {
    pub fn new(ctx: &Context) -> Result<Self> {
        let syncs = core::array::try_from_fn(|i| {
            sync_state::SyncState::new(ctx, &format!("sync_state_{i}"))
        })?;

        let handle = {
            let create_info = vk::SwapchainCreateInfoKHR::default()
                .surface(**ctx.surface)
                .min_image_count(ctx.surface.config.image_count)
                .image_format(surface::conf::FORMAT.format)
                .image_color_space(surface::conf::FORMAT.color_space)
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
                    .map_err(Error::Create)?
            }
        };

        let images = unsafe {
            ctx.ext
                .swapchain
                .get_swapchain_images(handle)
                .map_err(Error::GetSwapchainImages)?
        }
        .into_iter()
        .enumerate()
        .map(|(idx, image)| {
            image::Image::new(
                ctx,
                image,
                ctx.surface.config.extent,
                &format!("swapchain#{idx}"),
            )
        })
        .collect::<image::Result<_>>()?;

        Ok(Self {
            syncs,
            frame_idx: 0,
            images,
            handle,
        })
    }

    pub fn get_next_image(self, ctx: &Context, signal_to: vk::Semaphore) -> Result<u32> {
        match unsafe {
            ctx.ext
                .swapchain
                .acquire_next_image(self.handle, u64::MAX, signal_to, vk::Fence::null())
                .map_err(Error::AcquireNextImage)?
        } {
            (_, true) => Err(Error::NeedsRecreating),
            (image_index, false) => Ok(image_index),
        }
    }
}

impl Destroy<Context> for Swapchain {
    fn destroy_with(&mut self, ctx: &Context) {
        let Self {
            syncs,
            frame_idx: _,
            images,
            handle,
        } = self;

        syncs.destroy_with(ctx);
        images.destroy_with(ctx);
        unsafe {
            ctx.ext.swapchain.destroy_swapchain(*handle, None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create swapchain / {0}")]
    Create(vk::Result),
    #[error("failed to get swapchain images / {0}")]
    GetSwapchainImages(vk::Result),
    #[error("failed to acquire next image / {0}")]
    AcquireNextImage(vk::Result),
    #[error("needs recreating")]
    NeedsRecreating,
    #[error("image / {0}")]
    Image(#[from] image::Error),
    #[error("sync states / {0}")]
    SyncStates(#[from] sync_state::Error),
}

mod sync_state {
    use crate::{
        base::{fence, semaphore},
        context::Context,
        destroy::Destroy,
    };

    type Result<T> = core::result::Result<T, Error>;

    pub struct SyncState {
        available: semaphore::Semaphore,
        ready: semaphore::Semaphore,
        presented: fence::Fence,
    }

    impl SyncState {
        pub fn new(ctx: &Context, name_prefix: &str) -> Result<Self> {
            let available = semaphore::Semaphore::new(ctx, &format!("{name_prefix}:available"))?;
            let ready = semaphore::Semaphore::new(ctx, &format!("{name_prefix}:ready"))?;
            let presented = fence::Fence::new(ctx, true, &format!("{name_prefix}:ready"))?;

            Ok(Self {
                available,
                ready,
                presented,
            })
        }
    }

    impl Destroy<Context> for SyncState {
        fn destroy_with(&mut self, ctx: &Context) {
            let Self {
                available,
                ready,
                presented,
            } = self;

            available.destroy_with(ctx);
            ready.destroy_with(ctx);
            presented.destroy_with(ctx);
        }
    }

    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        #[error("fence / {0}")]
        Fence(#[from] fence::Error),
        #[error("semaphore / {0}")]
        Semaphore(#[from] semaphore::Error),
    }
}

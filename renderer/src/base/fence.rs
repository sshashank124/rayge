use ash::vk;

use crate::{
    context::{Context, device},
    destroy::Destroy,
};

type Result<T> = core::result::Result<T, Error>;

pub struct Fence {
    handle: vk::Fence,
}

impl Fence {
    pub fn new(ctx: &Context, signaled: bool, name: &str) -> Result<Self> {
        let handle = {
            let create_info = vk::FenceCreateInfo::default().flags(if signaled {
                vk::FenceCreateFlags::SIGNALED
            } else {
                vk::FenceCreateFlags::empty()
            });

            unsafe {
                ctx.create_fence(&create_info, None)
                    .map_err(Error::Create)?
            }
        };
        ctx.set_debug_name(handle, name)?;

        Ok(Self { handle })
    }
}

impl Destroy<Context> for Fence {
    fn destroy_with(&mut self, ctx: &Context) {
        let Self { handle } = self;
        unsafe {
            ctx.destroy_fence(*handle, None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create fence / {0}")]
    Create(vk::Result),
    #[error("device / {0}")]
    Device(#[from] device::Error),
}

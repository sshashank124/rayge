use ash::vk;

use crate::{
    context::{Context, device},
    destroy::Destroy,
};

type Result<T> = core::result::Result<T, Error>;

pub struct Semaphore {
    handle: vk::Semaphore,
}

impl Semaphore {
    pub fn new(ctx: &Context, name: &str) -> Result<Self> {
        let handle = {
            let create_info = vk::SemaphoreCreateInfo::default();

            unsafe {
                ctx.create_semaphore(&create_info, None)
                    .map_err(Error::Create)?
            }
        };
        ctx.set_debug_name(handle, name)?;

        Ok(Self { handle })
    }
}

impl Destroy<Context> for Semaphore {
    fn destroy_with(&mut self, ctx: &Context) {
        let Self { handle } = self;
        unsafe {
            ctx.destroy_semaphore(*handle, None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create semaphore / {0}")]
    Create(vk::Result),
    #[error("device / {0}")]
    Device(#[from] device::Error),
}

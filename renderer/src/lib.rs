#![feature(adt_const_params)]
#![feature(array_try_from_fn)]
#![feature(let_chains)]

use context::device;
use destroy::Destroy;
use swapchain::Swapchain;

mod base;
mod context;
mod destroy;
mod swapchain;

type Result<T> = core::result::Result<T, Error>;

pub struct Renderer {
    swapchain: swapchain::Swapchain,
    ctx: context::Context,
}

impl Renderer {
    pub fn new(window: &impl raw_window_handle::HasWindowHandle) -> Result<Self> {
        let ctx = context::Context::new(window)?;
        let swapchain = swapchain::Swapchain::new(&ctx)?;

        Ok(Self { swapchain, ctx })
    }

    pub fn render(&mut self) -> Result<bool> {
        Ok(false)
    }

    pub fn resize(&mut self) -> Result<bool> {
        let is_valid = self.ctx.refresh_surface_capabilities()?;
        if is_valid {
            self.ctx.wait_idle()?;
            self.swapchain.destroy_with(&self.ctx);
            self.swapchain = Swapchain::new(&self.ctx)?;
        }
        Ok(is_valid)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use destroy::Destroy;

        let Self { swapchain, ctx } = self;

        ctx.wait_idle().expect("Failed to wait for device to idle");
        swapchain.destroy_with(ctx);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("context / {0}")]
    Context(#[from] context::Error),
    #[error("device / {0}")]
    Device(#[from] device::Error),
    #[error("swapchain / {0}")]
    Swapchain(#[from] swapchain::Error),
}

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

pub type Result<T> = core::result::Result<T, Error>;

pub struct Renderer {
    swapchain: swapchain::Swapchain,
    needs_resizing: bool,
    ctx: context::Context,
}

impl Renderer {
    pub fn new(window: &impl raw_window_handle::HasWindowHandle) -> Result<Self> {
        let ctx = context::Context::new(window)?;
        let swapchain = swapchain::Swapchain::new(&ctx)?;

        Ok(Self {
            swapchain,
            needs_resizing: false,
            ctx,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        if self.needs_resizing && !self.resize()? {
            return Ok(());
        }
        Ok(())
    }

    pub const fn needs_resizing(&mut self) {
        self.needs_resizing = true;
    }

    fn resize(&mut self) -> Result<bool> {
        let is_valid = self.ctx.refresh_surface_capabilities()?;
        if is_valid {
            self.ctx.wait_idle()?;
            self.swapchain.destroy_with(&self.ctx);
            self.swapchain = Swapchain::new(&self.ctx)?;
            self.needs_resizing = false;
        }
        Ok(is_valid)
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        use destroy::Destroy;

        let Self {
            swapchain,
            needs_resizing: _,
            ctx,
        } = self;

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

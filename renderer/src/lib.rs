#![feature(let_chains)]

mod context;
mod swapchain;

use std::rc::Rc;

use raw_window_handle::HasWindowHandle;

use context::Context;
use swapchain::Swapchain;

type Ctx = Rc<Context>;

pub struct Renderer {
    swapchain: Swapchain,
    context: Ctx,
}

impl Renderer {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self, RendererError> {
        let context = Rc::new(Context::new(window)?);

        let swapchain = Swapchain::new(&context)?;

        Ok(Self { swapchain, context })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("context / {0}")]
    Context(#[from] context::ContextError),
    #[error("swapchain / {0}")]
    Swapchain(#[from] swapchain::SwapchainError),
}

#![feature(let_chains)]

mod context;

use context::Context;
use raw_window_handle::HasWindowHandle;

pub struct Renderer {
    _context: Context,
}

impl Renderer {
    pub fn new(window: &impl HasWindowHandle) -> Result<Self, RendererError> {
        let context = Context::new(window)?;

        Ok(Self { _context: context })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RendererError {
    #[error("context / {0}")]
    Context(#[from] context::ContextError),
}

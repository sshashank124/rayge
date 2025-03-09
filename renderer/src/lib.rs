mod context;

use context::Context;

pub struct Renderer {
    _context: Context,
}

impl Renderer {
    pub fn new() -> Result<Self, RendererCreateError> {
        let context = Context::new()?;

        Ok(Self { _context: context })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create Renderer")]
pub struct RendererCreateError(#[from] context::ContextCreateError);

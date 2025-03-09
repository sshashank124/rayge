mod extensions;
mod instance;

use instance::Instance;

pub struct Context {
    _instance: Instance,
}

impl Context {
    pub(super) fn new() -> Result<Self, ContextCreateError> {
        let instance = Instance::new()?;

        Ok(Self {
            _instance: instance,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to create Context")]
pub struct ContextCreateError(#[from] instance::InstanceCreateError);

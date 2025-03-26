use ash::{Entry, vk};

use super::extensions;

pub mod conf {
    pub const APPLICATION_NAME: &std::ffi::CStr = c"RAYGE Renderer";
    pub const VK_API_VERSION: u32 = ash::vk::API_VERSION_1_3;
}

type Result<T> = core::result::Result<T, InstanceError>;

pub struct Instance {
    instance: ash::Instance,
    pub entry: Entry,
}

impl Instance {
    pub fn new() -> Result<Self> {
        let entry = unsafe { Entry::load()? };

        let instance = {
            let app_info = vk::ApplicationInfo::default()
                .application_name(conf::APPLICATION_NAME)
                .api_version(conf::VK_API_VERSION);

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(extensions::instance::REQUIRED)
                .flags(extensions::instance::FLAGS);

            unsafe {
                entry
                    .create_instance(&create_info, None)
                    .map_err(InstanceError::Create)?
            }
        };

        Ok(Self { instance, entry })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

impl std::ops::Deref for Instance {
    type Target = ash::Instance;
    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceError {
    #[error("failed to load vulkan entry-point / {0}")]
    LoadEntry(#[from] ash::LoadingError),
    #[error("failed to create vulkan instance / {0}")]
    Create(vk::Result),
}

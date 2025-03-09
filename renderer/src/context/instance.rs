use ash::{Entry, vk};

use super::extensions;

mod conf {
    pub(super) const APPLICATION_NAME: &std::ffi::CStr = c"RAYGE Renderer";
    pub(super) const VK_API_VERSION: u32 = ash::vk::API_VERSION_1_3;
}

pub(super) struct Instance {
    instance: ash::Instance,
    _entry: Entry,
}

impl Instance {
    pub(super) fn new() -> Result<Self, InstanceCreateError> {
        let entry = unsafe { Entry::load()? };

        let instance = {
            let app_info = vk::ApplicationInfo::default()
                .application_name(conf::APPLICATION_NAME)
                .api_version(conf::VK_API_VERSION);

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(extensions::instance::REQUIRED)
                .flags(extensions::instance::FLAGS);

            unsafe { entry.create_instance(&create_info, None)? }
        };

        Ok(Self {
            instance,
            _entry: entry,
        })
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InstanceCreateError {
    #[error("failed to load Vulkan entry-point")]
    LoadEntry(#[from] ash::LoadingError),
    #[error("failed to create Vulkan instance")]
    Create(#[from] vk::Result),
}

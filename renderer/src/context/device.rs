use std::mem::ManuallyDrop;

use ash::vk;

use super::{
    extensions, features, instance, physical_device::PhysicalDevice, queue, surface::Surface,
};

type Result<T> = core::result::Result<T, Error>;

pub struct Device {
    queues: queue::Queues,
    pub ext: extensions::Handles,
    allocator: ManuallyDrop<vk_mem::Allocator>,
    handle: ash::Device,
}

impl Device {
    pub fn new(
        instance: &instance::Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Result<Self> {
        let (queue_create_infos, queue_families) =
            queue::Queues::create_infos(instance, physical_device, surface)?;

        let handle = {
            let (required_features, mut additional_required_features) = features::required();
            let mut required_features = additional_required_features
                .iter_mut()
                .fold(required_features, |acc_features, f| {
                    acc_features.push_next(f.as_mut())
                });

            let extensions_to_enable = extensions::device::REQUIRED
                .iter()
                .map(|e| e.as_ptr())
                .collect::<Vec<_>>();

            let create_info = vk::DeviceCreateInfo::default()
                .enabled_extension_names(&extensions_to_enable)
                .push_next(&mut required_features)
                .queue_create_infos(&queue_create_infos);

            unsafe {
                instance
                    .create_device(**physical_device, &create_info, None)
                    .map_err(Error::Create)?
            }
        };

        let allocator = {
            let mut create_info =
                vk_mem::AllocatorCreateInfo::new(instance, &handle, **physical_device);
            create_info.vulkan_api_version = instance::conf::VK_API_VERSION;
            create_info.flags = vk_mem::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION
                | vk_mem::AllocatorCreateFlags::KHR_BIND_MEMORY2
                | vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS
                | vk_mem::AllocatorCreateFlags::EXT_MEMORY_PRIORITY;

            ManuallyDrop::new(unsafe {
                vk_mem::Allocator::new(create_info).map_err(Error::Allocator)
            }?)
        };

        let ext = extensions::Handles::new(instance, &handle);

        let queues = queue::Queues::new(&handle, &queue_families);

        Ok(Self {
            queues,
            ext,
            allocator,
            handle,
        })
    }

    #[cfg(feature = "debug-names")]
    pub fn set_debug_name<H: vk::Handle>(&self, object: H, name: &str) -> Result<()> {
        let object_name = std::ffi::CString::new(name).unwrap();
        let name_info = vk::DebugUtilsObjectNameInfoEXT::default()
            .object_handle(object)
            .object_name(&object_name);

        unsafe {
            self.ext
                .debug_utils
                .set_debug_utils_object_name(&name_info)
                .map_err(Error::SetDebugName)
        }
    }

    #[cfg(not(feature = "debug-names"))]
    pub fn set_debug_name<H>(&self, _: H, _: &str) -> Result<()> {
        Ok(())
    }

    pub fn wait_idle(&self) -> Result<()> {
        unsafe { self.device_wait_idle().map_err(Error::WaitIdle) }
    }
}

impl std::ops::Deref for Device {
    type Target = ash::Device;
    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        let Self {
            queues: _,
            ext: _,
            allocator,
            handle,
        } = self;
        unsafe {
            ManuallyDrop::drop(allocator);
            handle.destroy_device(None);
        }
    }
}

impl core::fmt::Debug for Device {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self {
            queues,
            ext: _,
            allocator: _,
            handle: _,
        } = self;
        f.debug_struct("Device")
            .field("queues", queues)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to create device / {0}")]
    Create(vk::Result),
    #[error("queue / {0}")]
    Queue(#[from] queue::Error),
    #[error("allocator / {0}")]
    Allocator(vk::Result),
    #[error("failed to set debug name / {0}")]
    SetDebugName(vk::Result),
    #[error("failed to wait for device to idle / {0}")]
    WaitIdle(vk::Result),
}

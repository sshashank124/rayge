use std::mem::ManuallyDrop;

use ash::vk;

use super::{
    extensions, features,
    instance::{self, Instance},
    physical_device::PhysicalDevice,
    queue::{QueueError, Queues},
    surface::Surface,
};

type Result<T> = core::result::Result<T, DeviceError>;

pub struct Device {
    queues: Queues,
    ext: extensions::Handles,
    allocator: ManuallyDrop<vk_mem::Allocator>,
    device: ash::Device,
}

impl Device {
    pub fn new(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Result<Self> {
        let (queue_create_infos, queue_families) =
            Queues::create_infos(instance, physical_device, surface)?;

        let device = {
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
                    .map_err(DeviceError::Create)?
            }
        };

        let allocator = {
            let mut create_info =
                vk_mem::AllocatorCreateInfo::new(instance, &device, **physical_device);
            create_info.vulkan_api_version = instance::conf::VK_API_VERSION;
            create_info.flags = vk_mem::AllocatorCreateFlags::KHR_DEDICATED_ALLOCATION
                | vk_mem::AllocatorCreateFlags::KHR_BIND_MEMORY2
                | vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS
                | vk_mem::AllocatorCreateFlags::EXT_MEMORY_PRIORITY;

            ManuallyDrop::new(unsafe {
                vk_mem::Allocator::new(create_info).map_err(DeviceError::Allocator)
            }?)
        };

        let ext = extensions::Handles::new(instance, &device);

        let queues = Queues::new(&device, &queue_families);

        Ok(Self {
            queues,
            ext,
            allocator,
            device,
        })
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.allocator);
            self.device.destroy_device(None);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceError {
    #[error("failed to create device / {0}")]
    Create(vk::Result),
    #[error("queue / {0}")]
    Queue(#[from] QueueError),
    #[error("allocator / {0}")]
    Allocator(vk::Result),
}

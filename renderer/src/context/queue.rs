use std::collections::HashSet;

use ash::vk;

use super::{
    instance::Instance,
    physical_device::PhysicalDevice,
    surface::{Surface, SurfaceError},
};

type Result<T> = core::result::Result<T, QueueError>;

pub struct Queues {
    graphics: Queue,
    compute: Queue,
    transfer: Queue,
}

pub struct Queue {
    queue: vk::Queue,
    family: u32,
}

pub struct Families {
    graphics: u32,
    compute: u32,
    transfer: u32,
}

#[derive(Debug, Default)]
struct FamiliesInfo {
    graphics: Option<u32>,
    compute: Option<u32>,
    transfer: Option<u32>,
}

impl Queues {
    pub fn new(device: &ash::Device, families: &Families) -> Self {
        let graphics = Queue::new(device, families.graphics, 0);
        let compute = Queue::new(device, families.compute, 0);
        let transfer = Queue::new(device, families.transfer, 0);

        Self {
            graphics,
            compute,
            transfer,
        }
    }

    pub fn create_infos(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Result<(Vec<vk::DeviceQueueCreateInfo<'static>>, Families)> {
        let families = Families::create_infos(instance, physical_device, surface)?;

        let create_infos = families
            .unique()
            .iter()
            .map(|&index| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(index)
                    .queue_priorities(&[1.0_f32])
            })
            .collect::<Vec<_>>();

        Ok((create_infos, families))
    }
}

impl Queue {
    fn new(device: &ash::Device, family: u32, index: u32) -> Self {
        let queue = unsafe { device.get_device_queue(family, index) };

        Self { queue, family }
    }
}

impl Families {
    pub fn create_infos(
        instance: &Instance,
        physical_device: &PhysicalDevice,
        surface: &Surface,
    ) -> Result<Self> {
        let family_props =
            unsafe { instance.get_physical_device_queue_family_properties(**physical_device) }
                .into_iter()
                .enumerate()
                .filter(|(_, family)| family.queue_count > 0);

        let mut found_indices = FamiliesInfo::default();
        for (index, queue_family) in family_props {
            let idx = index as u32;

            let g = queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS);
            let c = queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE);
            let t = queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER);
            let vd = queue_family
                .queue_flags
                .contains(vk::QueueFlags::VIDEO_DECODE_KHR);
            let ve = queue_family
                .queue_flags
                .contains(vk::QueueFlags::VIDEO_ENCODE_KHR);

            if found_indices.graphics.is_none()
                && g
                && surface.is_supported_by(physical_device, idx)?
            {
                found_indices.graphics = Some(idx);
            } else if found_indices.compute.is_none() && c {
                if let Some(g_idx) = found_indices.graphics
                    && idx != g_idx
                {
                    found_indices.compute = Some(idx);
                }
            } else if found_indices.transfer.is_none() && t && !vd && !ve {
                found_indices.transfer = Some(idx);
            }

            if found_indices.is_complete() {
                break;
            }
        }

        Self::try_from(found_indices).map_err(QueueError::Create)
    }

    fn unique(&self) -> HashSet<u32> {
        HashSet::from([self.graphics, self.compute, self.transfer])
    }
}

impl FamiliesInfo {
    pub const fn is_complete(&self) -> bool {
        self.graphics.is_some() && self.compute.is_some() && self.transfer.is_some()
    }
}

impl TryFrom<FamiliesInfo> for Families {
    type Error = String;
    fn try_from(value: FamiliesInfo) -> core::result::Result<Self, Self::Error> {
        Ok(Self {
            graphics: value.graphics.ok_or("graphics")?,
            compute: value.compute.ok_or("compute")?,
            transfer: value.transfer.ok_or("compute")?,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum QueueError {
    #[error("failed to create {0} queue")]
    Create(String),
    #[error("surface error / {0}")]
    Surface(#[from] SurfaceError),
}

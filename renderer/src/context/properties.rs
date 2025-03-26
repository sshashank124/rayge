use ash::vk;

use super::instance::Instance;

#[derive(Debug)]
pub struct Properties {
    pub core: CoreProperties,
    pub acceleration_structure: AccelerationStructureProperties,
    pub ray_tracing_pipeline: RayTracingPipelineProperties,
}
#[derive(Debug)]
pub struct CoreProperties;
#[derive(Debug)]
pub struct AccelerationStructureProperties {
    pub min_scratch_offset_alignment: u32,
}
#[derive(Debug)]
pub struct RayTracingPipelineProperties {
    pub shader_group: ShaderGroupProperties,
}
#[derive(Debug)]
pub struct ShaderGroupProperties {
    pub base_alignment: u32,
    pub handle_alignment: u32,
    pub handle_size: u32,
}

impl Properties {
    pub fn get_supported(instance: &Instance, physical_device: vk::PhysicalDevice) -> Self {
        let mut ray_tracing_pipeline = vk::PhysicalDeviceRayTracingPipelinePropertiesKHR::default();
        let mut acceleration_structure =
            vk::PhysicalDeviceAccelerationStructurePropertiesKHR::default();

        let mut core = vk::PhysicalDeviceProperties2::default()
            .push_next(&mut ray_tracing_pipeline)
            .push_next(&mut acceleration_structure);

        unsafe { instance.get_physical_device_properties2(physical_device, &mut core) };

        Self {
            core: CoreProperties::from(core.properties),
            acceleration_structure: AccelerationStructureProperties::from(acceleration_structure),
            ray_tracing_pipeline: RayTracingPipelineProperties::from(ray_tracing_pipeline),
        }
    }
}

impl From<vk::PhysicalDeviceProperties> for CoreProperties {
    fn from(_p: vk::PhysicalDeviceProperties) -> Self {
        Self
    }
}

impl From<vk::PhysicalDeviceAccelerationStructurePropertiesKHR<'_>>
    for AccelerationStructureProperties
{
    fn from(p: vk::PhysicalDeviceAccelerationStructurePropertiesKHR) -> Self {
        Self {
            min_scratch_offset_alignment: p.min_acceleration_structure_scratch_offset_alignment,
        }
    }
}

impl From<vk::PhysicalDeviceRayTracingPipelinePropertiesKHR<'_>> for RayTracingPipelineProperties {
    fn from(p: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR) -> Self {
        Self {
            shader_group: From::from(p),
        }
    }
}

impl From<vk::PhysicalDeviceRayTracingPipelinePropertiesKHR<'_>> for ShaderGroupProperties {
    fn from(p: vk::PhysicalDeviceRayTracingPipelinePropertiesKHR) -> Self {
        Self {
            base_alignment: p.shader_group_base_alignment,
            handle_alignment: p.shader_group_handle_alignment,
            handle_size: p.shader_group_handle_size,
        }
    }
}

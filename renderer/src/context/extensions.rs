pub(super) mod instance {
    pub const REQUIRED: &[*const std::ffi::c_char] = &[
        ash::khr::portability_enumeration::NAME.as_ptr(),
        // Surface
        ash::khr::surface::NAME.as_ptr(),
    ];

    pub const FLAGS: ash::vk::InstanceCreateFlags =
        ash::vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
}

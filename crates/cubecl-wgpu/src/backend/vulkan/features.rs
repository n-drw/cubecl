use std::{ffi::CStr, ptr::null_mut};

use ash::vk::{
    DeviceCreateInfo, EXT_SHADER_ATOMIC_FLOAT_NAME, EXT_SHADER_ATOMIC_FLOAT2_NAME,
    ExtendsDeviceCreateInfo, ExtendsPhysicalDeviceFeatures2, KHR_COOPERATIVE_MATRIX_NAME,
    KHR_SHADER_FLOAT_CONTROLS2_NAME, PhysicalDevice8BitStorageFeatures,
    PhysicalDevice16BitStorageFeatures, PhysicalDeviceCooperativeMatrixFeaturesKHR,
    PhysicalDeviceFeatures2, PhysicalDeviceShaderAtomicFloat2FeaturesEXT,
    PhysicalDeviceShaderAtomicFloatFeaturesEXT, PhysicalDeviceShaderFloat16Int8Features,
    PhysicalDeviceShaderFloatControls2FeaturesKHR,
    PhysicalDeviceShaderSubgroupExtendedTypesFeatures, PhysicalDeviceVulkanMemoryModelFeatures,
};
use wgpu::{Features, hal::vulkan};

#[derive(Default, Debug)]
pub struct ExtendedFeatures<'a> {
    pub mem_model: PhysicalDeviceVulkanMemoryModelFeatures<'a>,
    pub float16_int8: PhysicalDeviceShaderFloat16Int8Features<'a>,
    pub buf_16: PhysicalDevice16BitStorageFeatures<'a>,
    pub buf_8: PhysicalDevice8BitStorageFeatures<'a>,
    pub subgroup_extended: PhysicalDeviceShaderSubgroupExtendedTypesFeatures<'a>,

    // extensions
    pub cmma: Option<PhysicalDeviceCooperativeMatrixFeaturesKHR<'a>>,
    pub atomic_float: Option<PhysicalDeviceShaderAtomicFloatFeaturesEXT<'a>>,
    pub atomic_float2: Option<PhysicalDeviceShaderAtomicFloat2FeaturesEXT<'a>>,
    pub float_controls2: Option<PhysicalDeviceShaderFloatControls2FeaturesKHR<'a>>,

    pub extensions: Vec<&'static CStr>,
}

impl<'a> ExtendedFeatures<'a> {
    pub fn from_adapter(
        ash: &ash::Instance,
        adapter: &vulkan::Adapter,
        features: Features,
    ) -> Self {
        let mut this = Self::default();
        this.fill_extensions(adapter, features);
        this.fill_features(ash, adapter);
        this
    }

    fn fill_extensions(&mut self, adapter: &vulkan::Adapter, features: Features) {
        self.extensions = adapter.required_device_extensions(features);
        let phys_caps = adapter.physical_device_capabilities();

        if phys_caps.supports_extension(KHR_COOPERATIVE_MATRIX_NAME) {
            self.extensions.push(KHR_COOPERATIVE_MATRIX_NAME);
            self.cmma = Some(PhysicalDeviceCooperativeMatrixFeaturesKHR::default())
        }

        if phys_caps.supports_extension(EXT_SHADER_ATOMIC_FLOAT_NAME) {
            self.extensions.push(EXT_SHADER_ATOMIC_FLOAT_NAME);
            self.atomic_float = Some(PhysicalDeviceShaderAtomicFloatFeaturesEXT::default());
        }

        if phys_caps.supports_extension(EXT_SHADER_ATOMIC_FLOAT2_NAME) {
            self.extensions.push(EXT_SHADER_ATOMIC_FLOAT2_NAME);
            self.atomic_float2 = Some(PhysicalDeviceShaderAtomicFloat2FeaturesEXT::default());
        }

        if phys_caps.supports_extension(KHR_SHADER_FLOAT_CONTROLS2_NAME) {
            self.extensions.push(KHR_SHADER_FLOAT_CONTROLS2_NAME);
            self.float_controls2 = Some(PhysicalDeviceShaderFloatControls2FeaturesKHR::default());
        }
    }

    pub fn add_to_device_create(&'a mut self, info: DeviceCreateInfo<'a>) -> DeviceCreateInfo<'a> {
        let mut info = info
            .push_next(&mut self.mem_model)
            .push_next(&mut self.float16_int8)
            .push_next(&mut self.buf_16)
            .push_next(&mut self.buf_8)
            .push_next(&mut self.subgroup_extended);

        fn push_opt<'a, T: ExtendsDeviceCreateInfo>(
            mut info: DeviceCreateInfo<'a>,
            feat: &'a mut Option<T>,
        ) -> DeviceCreateInfo<'a> {
            if let Some(feat) = feat {
                info = info.push_next(feat);
            }
            info
        }

        info = push_opt(info, &mut self.cmma);
        info = push_opt(info, &mut self.atomic_float);
        info = push_opt(info, &mut self.atomic_float2);
        info = push_opt(info, &mut self.float_controls2);

        info
    }

    fn fill_features(&mut self, ash: &ash::Instance, adapter: &vulkan::Adapter) {
        let mut features = PhysicalDeviceFeatures2::default()
            .push_next(&mut self.mem_model)
            .push_next(&mut self.float16_int8)
            .push_next(&mut self.buf_16)
            .push_next(&mut self.buf_8)
            .push_next(&mut self.subgroup_extended);

        fn push_opt<'a, T: ExtendsPhysicalDeviceFeatures2>(
            mut features: PhysicalDeviceFeatures2<'a>,
            feat: &'a mut Option<T>,
        ) -> PhysicalDeviceFeatures2<'a> {
            if let Some(feat) = feat {
                features = features.push_next(feat);
            }
            features
        }

        features = push_opt(features, &mut self.cmma);
        features = push_opt(features, &mut self.atomic_float);
        features = push_opt(features, &mut self.atomic_float2);
        features = push_opt(features, &mut self.float_controls2);

        unsafe {
            ash.get_physical_device_features2(adapter.raw_physical_device(), &mut features);
        }

        self.zero_pointers();
    }

    /// Leaving these set seems to cause misaligned deref
    fn zero_pointers(&mut self) {
        self.mem_model.p_next = null_mut();
        self.float16_int8.p_next = null_mut();
        self.buf_16.p_next = null_mut();
        self.buf_8.p_next = null_mut();
        self.subgroup_extended.p_next = null_mut();

        if let Some(cmma) = &mut self.cmma {
            cmma.p_next = null_mut();
        }
        if let Some(atomic_float) = &mut self.atomic_float {
            atomic_float.p_next = null_mut();
        }
        if let Some(atomic_float2) = &mut self.atomic_float2 {
            atomic_float2.p_next = null_mut();
        }
        if let Some(float_controls2) = &mut self.float_controls2 {
            float_controls2.p_next = null_mut();
        }
    }
}

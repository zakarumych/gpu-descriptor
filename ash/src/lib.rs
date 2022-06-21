use {
    ash::{vk, Device},
    gpu_descriptor_types::{
        CreatePoolError, DescriptorDevice, DescriptorPoolCreateFlags, DescriptorTotalCount,
        DeviceAllocationError,
    },
};

#[repr(transparent)]
pub struct AshDescriptorDevice {
    device: Device,
}

impl AshDescriptorDevice {
    pub fn wrap(device: &Device) -> &Self {
        unsafe {
            // Safe because `Self` is `repr(transparent)`
            // with only non-zero-sized field being `D`.
            &*(device as *const Device as *const Self)
        }
    }
}

impl DescriptorDevice<vk::DescriptorSetLayout, vk::DescriptorPool, vk::DescriptorSet>
    for AshDescriptorDevice
{
    unsafe fn create_descriptor_pool(
        &self,
        descriptor_count: &DescriptorTotalCount,
        max_sets: u32,
        flags: DescriptorPoolCreateFlags,
    ) -> Result<vk::DescriptorPool, CreatePoolError> {
        let mut array = [vk::DescriptorPoolSize::builder().build(); 13];
        let mut len = 0;

        if descriptor_count.sampler != 0 {
            array[len].ty = vk::DescriptorType::SAMPLER;
            array[len].descriptor_count = descriptor_count.sampler;
            len += 1;
        }

        if descriptor_count.combined_image_sampler != 0 {
            array[len].ty = vk::DescriptorType::COMBINED_IMAGE_SAMPLER;
            array[len].descriptor_count = descriptor_count.combined_image_sampler;
            len += 1;
        }

        if descriptor_count.sampled_image != 0 {
            array[len].ty = vk::DescriptorType::SAMPLED_IMAGE;
            array[len].descriptor_count = descriptor_count.sampled_image;
            len += 1;
        }

        if descriptor_count.storage_image != 0 {
            array[len].ty = vk::DescriptorType::STORAGE_IMAGE;
            array[len].descriptor_count = descriptor_count.storage_image;
            len += 1;
        }

        if descriptor_count.uniform_texel_buffer != 0 {
            array[len].ty = vk::DescriptorType::UNIFORM_TEXEL_BUFFER;
            array[len].descriptor_count = descriptor_count.uniform_texel_buffer;
            len += 1;
        }

        if descriptor_count.storage_texel_buffer != 0 {
            array[len].ty = vk::DescriptorType::STORAGE_TEXEL_BUFFER;
            array[len].descriptor_count = descriptor_count.storage_texel_buffer;
            len += 1;
        }

        if descriptor_count.uniform_buffer != 0 {
            array[len].ty = vk::DescriptorType::UNIFORM_BUFFER;
            array[len].descriptor_count = descriptor_count.uniform_buffer;
            len += 1;
        }

        if descriptor_count.storage_buffer != 0 {
            array[len].ty = vk::DescriptorType::STORAGE_BUFFER;
            array[len].descriptor_count = descriptor_count.storage_buffer;
            len += 1;
        }

        if descriptor_count.uniform_buffer_dynamic != 0 {
            array[len].ty = vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC;
            array[len].descriptor_count = descriptor_count.uniform_buffer_dynamic;
            len += 1;
        }

        if descriptor_count.storage_buffer_dynamic != 0 {
            array[len].ty = vk::DescriptorType::STORAGE_BUFFER_DYNAMIC;
            array[len].descriptor_count = descriptor_count.storage_buffer_dynamic;
            len += 1;
        }

        if descriptor_count.input_attachment != 0 {
            array[len].ty = vk::DescriptorType::INPUT_ATTACHMENT;
            array[len].descriptor_count = descriptor_count.input_attachment;
            len += 1;
        }

        if descriptor_count.acceleration_structure != 0 {
            array[len].ty = vk::DescriptorType::ACCELERATION_STRUCTURE_KHR;
            array[len].descriptor_count = descriptor_count.acceleration_structure;
            len += 1;
        }

        if descriptor_count.inline_uniform_block_bytes != 0 {
            panic!("Inline uniform blocks are not supported");
        }

        if descriptor_count.inline_uniform_block_bindings != 0 {
            panic!("Inline uniform blocks are not supported");
        }

        let mut ash_flags = vk::DescriptorPoolCreateFlags::empty();

        if flags.contains(DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET) {
            ash_flags |= vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;
        }

        if flags.contains(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND) {
            ash_flags |= vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND;
        }

        let result = self.device.create_descriptor_pool(
            &vk::DescriptorPoolCreateInfo::builder()
                .max_sets(max_sets)
                .pool_sizes(&array[..len])
                .flags(ash_flags),
            None,
        );

        match result {
            Ok(pool) => Ok(pool),
            Err(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY) => Err(CreatePoolError::OutOfDeviceMemory),
            Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY) => Err(CreatePoolError::OutOfHostMemory),
            Err(vk::Result::ERROR_FRAGMENTATION) => Err(CreatePoolError::Fragmentation),
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }

    unsafe fn destroy_descriptor_pool(&self, pool: vk::DescriptorPool) {
        self.device.destroy_descriptor_pool(pool, None)
    }

    unsafe fn alloc_descriptor_sets<'a>(
        &self,
        pool: &mut vk::DescriptorPool,
        layouts: impl ExactSizeIterator<Item = &'a vk::DescriptorSetLayout>,
        sets: &mut impl Extend<vk::DescriptorSet>,
    ) -> Result<(), DeviceAllocationError> {
        let set_layouts: smallvec::SmallVec<[_; 16]> = layouts.copied().collect();

        match self.device.allocate_descriptor_sets(
            &vk::DescriptorSetAllocateInfo::builder()
                .set_layouts(&set_layouts)
                .descriptor_pool(*pool),
        ) {
            Ok(allocated) => {
                sets.extend(allocated);
                Ok(())
            }
            Err(vk::Result::ERROR_OUT_OF_HOST_MEMORY) => {
                Err(DeviceAllocationError::OutOfHostMemory)
            }
            Err(vk::Result::ERROR_OUT_OF_DEVICE_MEMORY) => {
                Err(DeviceAllocationError::OutOfDeviceMemory)
            }
            Err(vk::Result::ERROR_FRAGMENTED_POOL) => Err(DeviceAllocationError::OutOfPoolMemory),
            Err(vk::Result::ERROR_OUT_OF_POOL_MEMORY) => Err(DeviceAllocationError::FragmentedPool),
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }

    unsafe fn dealloc_descriptor_sets<'a>(
        &self,
        pool: &mut vk::DescriptorPool,
        sets: impl Iterator<Item = vk::DescriptorSet>,
    ) {
        let sets: smallvec::SmallVec<[_; 16]> = sets.collect();
        match self.device.free_descriptor_sets(*pool, &sets) {
            Ok(()) => {}
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }
}

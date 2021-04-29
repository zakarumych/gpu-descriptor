use {
    erupt::{vk1_0, DeviceLoader},
    gpu_descriptor_types::{
        CreatePoolError, DescriptorDevice, DescriptorPoolCreateFlags, DescriptorTotalCount,
        DeviceAllocationError,
    },
};

#[repr(transparent)]
pub struct EruptDescriptorDevice {
    device: DeviceLoader,
}

impl EruptDescriptorDevice {
    pub fn wrap(device: &DeviceLoader) -> &Self {
        unsafe {
            // Safe because `Self` is `repr(transparent)`
            // with only non-zero-sized field being `D`.
            &*(device as *const DeviceLoader as *const Self)
        }
    }
}

impl DescriptorDevice<vk1_0::DescriptorSetLayout, vk1_0::DescriptorPool, vk1_0::DescriptorSet>
    for EruptDescriptorDevice
{
    unsafe fn create_descriptor_pool(
        &self,
        descriptor_count: &DescriptorTotalCount,
        max_sets: u32,
        flags: DescriptorPoolCreateFlags,
    ) -> Result<vk1_0::DescriptorPool, CreatePoolError> {
        let mut array = [vk1_0::DescriptorPoolSizeBuilder::default(); 13];
        let mut len = 0;

        if descriptor_count.sampler != 0 {
            array[len]._type = vk1_0::DescriptorType::SAMPLER;
            array[len].descriptor_count = descriptor_count.sampler;
            len += 1;
        }

        if descriptor_count.combined_image_sampler != 0 {
            array[len]._type = vk1_0::DescriptorType::COMBINED_IMAGE_SAMPLER;
            array[len].descriptor_count = descriptor_count.combined_image_sampler;
            len += 1;
        }

        if descriptor_count.sampled_image != 0 {
            array[len]._type = vk1_0::DescriptorType::SAMPLED_IMAGE;
            array[len].descriptor_count = descriptor_count.sampled_image;
            len += 1;
        }

        if descriptor_count.storage_image != 0 {
            array[len]._type = vk1_0::DescriptorType::STORAGE_IMAGE;
            array[len].descriptor_count = descriptor_count.storage_image;
            len += 1;
        }

        if descriptor_count.uniform_texel_buffer != 0 {
            array[len]._type = vk1_0::DescriptorType::UNIFORM_TEXEL_BUFFER;
            array[len].descriptor_count = descriptor_count.uniform_texel_buffer;
            len += 1;
        }

        if descriptor_count.storage_texel_buffer != 0 {
            array[len]._type = vk1_0::DescriptorType::STORAGE_TEXEL_BUFFER;
            array[len].descriptor_count = descriptor_count.storage_texel_buffer;
            len += 1;
        }

        if descriptor_count.uniform_buffer != 0 {
            array[len]._type = vk1_0::DescriptorType::UNIFORM_BUFFER;
            array[len].descriptor_count = descriptor_count.uniform_buffer;
            len += 1;
        }

        if descriptor_count.storage_buffer != 0 {
            array[len]._type = vk1_0::DescriptorType::STORAGE_BUFFER;
            array[len].descriptor_count = descriptor_count.storage_buffer;
            len += 1;
        }

        if descriptor_count.uniform_buffer_dynamic != 0 {
            array[len]._type = vk1_0::DescriptorType::UNIFORM_BUFFER_DYNAMIC;
            array[len].descriptor_count = descriptor_count.uniform_buffer_dynamic;
            len += 1;
        }

        if descriptor_count.storage_buffer_dynamic != 0 {
            array[len]._type = vk1_0::DescriptorType::STORAGE_BUFFER_DYNAMIC;
            array[len].descriptor_count = descriptor_count.storage_buffer_dynamic;
            len += 1;
        }

        if descriptor_count.input_attachment != 0 {
            array[len]._type = vk1_0::DescriptorType::INPUT_ATTACHMENT;
            array[len].descriptor_count = descriptor_count.input_attachment;
            len += 1;
        }

        if descriptor_count.acceleration_structure != 0 {
            panic!("Acceleration structures are not supported");
        }

        if descriptor_count.inline_uniform_block_bytes != 0 {
            panic!("Inline uniform blocks are not supported");
        }

        if descriptor_count.inline_uniform_block_bindings != 0 {
            panic!("Inline uniform blocks are not supported");
        }

        let mut erupt_flags = vk1_0::DescriptorPoolCreateFlags::empty();

        if flags.contains(DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET) {
            erupt_flags |= vk1_0::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;
        }

        if flags.contains(DescriptorPoolCreateFlags::UPDATE_AFTER_BIND) {
            erupt_flags |= vk1_0::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND;
        }

        let result = self
            .device
            .create_descriptor_pool(
                &vk1_0::DescriptorPoolCreateInfoBuilder::default()
                    .max_sets(max_sets)
                    .pool_sizes(&array[..len])
                    .flags(erupt_flags)
                    .build(),
                None,
                None,
            )
            .result();

        match result {
            Ok(pool) => Ok(pool),
            Err(vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY) => {
                Err(CreatePoolError::OutOfDeviceMemory)
            }
            Err(vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY) => Err(CreatePoolError::OutOfHostMemory),
            Err(vk1_0::Result::ERROR_FRAGMENTATION) => Err(CreatePoolError::Fragmentation),
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }

    unsafe fn destroy_descriptor_pool(&self, pool: vk1_0::DescriptorPool) {
        self.device.destroy_descriptor_pool(Some(pool), None)
    }

    unsafe fn alloc_descriptor_sets<'a>(
        &self,
        pool: &mut vk1_0::DescriptorPool,
        layouts: impl ExactSizeIterator<Item = &'a vk1_0::DescriptorSetLayout>,
        sets: &mut impl Extend<vk1_0::DescriptorSet>,
    ) -> Result<(), DeviceAllocationError> {
        let set_layouts: smallvec::SmallVec<[_; 16]> = layouts.copied().collect();

        match self
            .device
            .allocate_descriptor_sets(
                &vk1_0::DescriptorSetAllocateInfoBuilder::default()
                    .set_layouts(&set_layouts)
                    .descriptor_pool(*pool),
            )
            .result()
        {
            Ok(allocated) => {
                sets.extend(allocated);
                Ok(())
            }
            Err(vk1_0::Result::ERROR_OUT_OF_HOST_MEMORY) => {
                Err(DeviceAllocationError::OutOfHostMemory)
            }
            Err(vk1_0::Result::ERROR_OUT_OF_DEVICE_MEMORY) => {
                Err(DeviceAllocationError::OutOfDeviceMemory)
            }
            Err(vk1_0::Result::ERROR_FRAGMENTED_POOL) => {
                Err(DeviceAllocationError::OutOfPoolMemory)
            }
            Err(vk1_0::Result::ERROR_OUT_OF_POOL_MEMORY) => {
                Err(DeviceAllocationError::FragmentedPool)
            }
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }

    unsafe fn dealloc_descriptor_sets<'a>(
        &self,
        pool: &mut vk1_0::DescriptorPool,
        sets: impl Iterator<Item = vk1_0::DescriptorSet>,
    ) {
        let sets: smallvec::SmallVec<[_; 16]> = sets.collect();
        match self.device.free_descriptor_sets(*pool, &sets).result() {
            Ok(()) => {}
            Err(err) => panic!("Unexpected return code '{}'", err),
        }
    }
}

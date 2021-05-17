use {
    gfx_hal::{
        device::{Device, OutOfMemory},
        pso::{
            self, BufferDescriptorFormat, BufferDescriptorType, DescriptorPool as _,
            DescriptorRangeDesc, DescriptorType, ImageDescriptorType,
        },
        Backend,
    },
    gpu_descriptor_types::{
        CreatePoolError, DescriptorDevice, DescriptorPoolCreateFlags, DescriptorTotalCount,
        DeviceAllocationError,
    },
    std::convert::TryFrom as _,
};

#[repr(transparent)]
pub struct GfxDescriptorDevice<B: Backend> {
    device: B::Device,
}

impl<B> GfxDescriptorDevice<B>
where
    B: Backend,
{
    pub fn wrap<D>(device: &D) -> &Self
    where
        D: Device<B>,
        B: Backend<Device = D>,
    {
        unsafe {
            // Safe because `Self` is `repr(transparent)`
            // with only non-zero-sized field being `D`.
            &*(device as *const D as *const Self)
        }
    }
}

impl<B> DescriptorDevice<B::DescriptorSetLayout, B::DescriptorPool, B::DescriptorSet>
    for GfxDescriptorDevice<B>
where
    B: Backend,
{
    unsafe fn create_descriptor_pool(
        &self,
        descriptor_count: &DescriptorTotalCount,
        max_sets: u32,
        flags: DescriptorPoolCreateFlags,
    ) -> Result<B::DescriptorPool, CreatePoolError> {
        assert!(
            KNOWN_CREATE_POOL_FLAGS.contains(flags),
            "Following flags specified but not supported by gfx backend: `{:#?}`",
            flags - KNOWN_CREATE_POOL_FLAGS
        );

        let count_cvt =
            |count: u32| usize::try_from(count).map_err(|_| CreatePoolError::OutOfHostMemory);

        let max_sets = count_cvt(max_sets)?;

        let mut array = [DescriptorRangeDesc {
            ty: DescriptorType::Sampler,
            count: 0,
        }; 13];
        let mut len = 0;

        if descriptor_count.sampler != 0 {
            array[len].ty = DescriptorType::Sampler;
            array[len].count = count_cvt(descriptor_count.sampler)?;
            len += 1;
        }

        if descriptor_count.combined_image_sampler != 0 {
            array[len].ty = DescriptorType::Image {
                ty: ImageDescriptorType::Sampled { with_sampler: true },
            };
            array[len].count = count_cvt(descriptor_count.combined_image_sampler)?;
            len += 1;
        }

        if descriptor_count.sampled_image != 0 {
            array[len].ty = DescriptorType::Image {
                ty: ImageDescriptorType::Sampled {
                    with_sampler: false,
                },
            };
            array[len].count = count_cvt(descriptor_count.sampled_image)?;
            len += 1;
        }

        if descriptor_count.storage_image != 0 {
            array[len].ty = DescriptorType::Image {
                ty: ImageDescriptorType::Storage { read_only: false },
            };
            array[len].count = count_cvt(descriptor_count.storage_image)?;
            len += 1;
        }

        if descriptor_count.uniform_texel_buffer != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Texel,
            };
            array[len].count = count_cvt(descriptor_count.uniform_texel_buffer)?;
            len += 1;
        }

        if descriptor_count.storage_texel_buffer != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Storage { read_only: false },
                format: BufferDescriptorFormat::Texel,
            };
            array[len].count = count_cvt(descriptor_count.storage_texel_buffer)?;
            len += 1;
        }

        if descriptor_count.uniform_buffer != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            };
            array[len].count = count_cvt(descriptor_count.uniform_buffer)?;
            len += 1;
        }

        if descriptor_count.storage_buffer != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Storage { read_only: false },
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: false,
                },
            };
            array[len].count = count_cvt(descriptor_count.storage_buffer)?;
            len += 1;
        }

        if descriptor_count.uniform_buffer_dynamic != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Uniform,
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: true,
                },
            };
            array[len].count = count_cvt(descriptor_count.uniform_buffer_dynamic)?;
            len += 1;
        }

        if descriptor_count.storage_buffer_dynamic != 0 {
            array[len].ty = DescriptorType::Buffer {
                ty: BufferDescriptorType::Storage { read_only: false },
                format: BufferDescriptorFormat::Structured {
                    dynamic_offset: true,
                },
            };
            array[len].count = count_cvt(descriptor_count.storage_buffer_dynamic)?;
            len += 1;
        }

        if descriptor_count.input_attachment != 0 {
            array[len].ty = DescriptorType::InputAttachment;
            array[len].count = count_cvt(descriptor_count.input_attachment)?;
            len += 1;
        }

        if len == 0 {
            array[0].ty = DescriptorType::Sampler;
            array[0].count = 1;
            len = 1;
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

        let mut gfx_flags = pso::DescriptorPoolCreateFlags::empty();

        if flags.contains(DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET) {
            gfx_flags |= pso::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;
        }

        let result =
            self.device
                .create_descriptor_pool(max_sets, array[..len].iter().cloned(), gfx_flags);

        match result {
            Ok(pool) => Ok(pool),
            Err(OutOfMemory::Host) => Err(CreatePoolError::OutOfHostMemory),
            Err(OutOfMemory::Device) => Err(CreatePoolError::OutOfDeviceMemory),
        }
    }

    unsafe fn destroy_descriptor_pool(&self, pool: B::DescriptorPool) {
        self.device.destroy_descriptor_pool(pool)
    }

    unsafe fn alloc_descriptor_sets<'a>(
        &self,
        pool: &mut B::DescriptorPool,
        layouts: impl ExactSizeIterator<Item = &'a B::DescriptorSetLayout>,
        sets: &mut impl Extend<B::DescriptorSet>,
    ) -> Result<(), DeviceAllocationError> {
        match pool.allocate(layouts, sets) {
            Ok(()) => Ok(()),
            Err(pso::AllocationError::OutOfMemory(OutOfMemory::Host)) => {
                Err(DeviceAllocationError::OutOfHostMemory)
            }
            Err(pso::AllocationError::OutOfMemory(OutOfMemory::Device)) => {
                Err(DeviceAllocationError::OutOfDeviceMemory)
            }
            Err(pso::AllocationError::OutOfPoolMemory) => {
                Err(DeviceAllocationError::OutOfPoolMemory)
            }
            Err(pso::AllocationError::FragmentedPool) => Err(DeviceAllocationError::FragmentedPool),
            Err(pso::AllocationError::IncompatibleLayout) => {
                panic!("Unexpected error `IncompatibleLayout`")
            }
        }
    }

    unsafe fn dealloc_descriptor_sets<'a>(
        &self,
        pool: &mut B::DescriptorPool,
        sets: impl Iterator<Item = B::DescriptorSet>,
    ) {
        pool.free(sets);
    }
}

/// Set of recognized flags.
/// Must include explicitly ignored flags as well.
const KNOWN_CREATE_POOL_FLAGS: DescriptorPoolCreateFlags =
    DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET;

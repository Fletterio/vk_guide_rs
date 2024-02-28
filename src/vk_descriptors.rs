use ash::vk;
use ash::Device;

pub struct DescriptorSetLayoutBuilder {
    pub bindings: Vec<vk::DescriptorSetLayoutBinding>,
}

impl DescriptorSetLayoutBuilder {
    pub fn add_binding(&mut self, binding: u32, descriptor_type: vk::DescriptorType) {
        self.bindings.push(
            vk::DescriptorSetLayoutBinding::builder()
                .binding(binding)
                .descriptor_count(1)
                .descriptor_type(descriptor_type)
                .build(),
        );
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.bindings.clear();
    }

    pub fn build(
        mut self,
        device: &Device,
        shader_stages: vk::ShaderStageFlags,
    ) -> vk::DescriptorSetLayout {
        for binding in self.bindings.iter_mut() {
            binding.stage_flags |= shader_stages;
        }
        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&self.bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::empty())
            .build();
        unsafe { device.create_descriptor_set_layout(&info, None).unwrap() }
    }
}

pub struct PoolSizeRatio {
    pub descriptor_type: vk::DescriptorType,
    pub ratio: f32,
}
#[derive(Default)]
pub struct DescriptorAllocator {
    pub pool: vk::DescriptorPool,
}

impl DescriptorAllocator {
    pub fn init_pool(&mut self, device: &Device, max_sets: u32, pool_ratios: &[PoolSizeRatio]) {
        let mut pool_sizes: Vec<vk::DescriptorPoolSize> = Vec::new();
        for ratio in pool_ratios {
            pool_sizes.push(
                vk::DescriptorPoolSize::builder()
                    .ty(ratio.descriptor_type)
                    .descriptor_count((ratio.ratio * max_sets as f32).floor() as u32)
                    .build(),
            );
        }
        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::empty())
            .max_sets(max_sets)
            .pool_sizes(&pool_sizes)
            .build();

        self.pool = unsafe { device.create_descriptor_pool(&pool_info, None).unwrap() };
    }

    #[allow(dead_code)]
    pub fn clear_descriptors(&mut self, device: &Device) {
        unsafe {
            device
                .reset_descriptor_pool(self.pool, vk::DescriptorPoolResetFlags::empty())
                .unwrap()
        };
    }

    #[allow(dead_code)]
    pub fn destroy_pool(&mut self, device: &Device) {
        unsafe { device.destroy_descriptor_pool(self.pool, None) };
    }

    //allocates only one descriptor
    pub fn allocate(
        &mut self,
        device: &Device,
        layout: vk::DescriptorSetLayout,
    ) -> vk::DescriptorSet {
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(std::slice::from_ref(&layout))
            .build();

        unsafe { device.allocate_descriptor_sets(&alloc_info).unwrap()[0] }
    }
}

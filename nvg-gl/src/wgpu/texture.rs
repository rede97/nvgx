use nvg::ImageFlags;
use wgpu::Extent3d;

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        image_flags: nvg::ImageFlags,
        texture_type: nvg::TextureType,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = match texture_type {
            nvg::TextureType::RGBA => {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("RGBA Texture"),
                    size,
                    mip_level_count: 1, // mipmap not supported yet
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            }
            nvg::TextureType::Alpha => {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("Alpha Texture"),
                    size,
                    mip_level_count: 1, // mipmap not supported yet
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::R8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            }
        };

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: if image_flags.contains(ImageFlags::REPEATX) {
                wgpu::AddressMode::Repeat
            } else {
                wgpu::AddressMode::ClampToEdge
            },
            address_mode_v: if image_flags.contains(ImageFlags::REPEATY) {
                wgpu::AddressMode::Repeat
            } else {
                wgpu::AddressMode::ClampToEdge
            },
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: if image_flags.contains(ImageFlags::NEAREST) {
                wgpu::FilterMode::Nearest
            } else {
                wgpu::FilterMode::Linear
            },
            min_filter: if image_flags.contains(ImageFlags::NEAREST) {
                wgpu::FilterMode::Nearest
            } else {
                wgpu::FilterMode::Linear
            },
            mipmap_filter: if image_flags.contains(ImageFlags::NEAREST) {
                wgpu::FilterMode::Nearest
            } else {
                wgpu::FilterMode::Linear
            },
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            view,
            sampler,
            bind_group,
        }
    }

    pub fn update(
        &self,
        queue: &wgpu::Queue,
        data: &[u8],
        origin: wgpu::Origin2d,
        size: wgpu::Extent3d,
    ) {
        let bytes_per_row = match self.texture.format() {
            wgpu::TextureFormat::Rgba8UnormSrgb => 4,
            wgpu::TextureFormat::R8Unorm => 1,
            _ => panic!("Unsupported texture format"),
        } * size.width;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 1,
                origin: origin.to_3d(0),
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(size.height),
            },
            size,
        );
    }

    #[inline]
    pub fn size(&self) -> Extent3d {
        self.texture.size()
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}

use nvgx::{ImageFlags, TextureType};
use slab::Slab;

#[allow(unused)]
pub struct StencilTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl StencilTexture {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let desc = wgpu::TextureDescriptor {
            label: Some("NVG Stencil Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Stencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        return Self { texture, view };
    }
}

#[allow(unused)]
pub struct Texture {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) sampler: wgpu::Sampler,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) image_flags: nvgx::ImageFlags,
    pub(crate) texture_type: nvgx::TextureType,
}

impl Texture {
    pub fn placeholder_texture(
        device: &wgpu::Device,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        return Self::new(
            device,
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            ImageFlags::empty(),
            nvgx::TextureType::RGBA,
            texture_bind_group_layout,
        );
    }

    pub fn new(
        device: &wgpu::Device,
        size: wgpu::Extent3d,
        image_flags: nvgx::ImageFlags,
        texture_type: nvgx::TextureType,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let texture = match texture_type {
            nvgx::TextureType::RGBA => {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("NVG RGBA Texture"),
                    size,
                    mip_level_count: 1, // mipmap not supported yet
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
            }
            nvgx::TextureType::BGRA => {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("NVG RGBA Texture"),
                    size,
                    mip_level_count: 1, // mipmap not supported yet
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                })
            }
            nvgx::TextureType::Alpha => {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("NVG Alpha Texture"),
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
            label: Some("NVG Sampler"),
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
            label: Some("NVG Texture Bind Group"),
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
            image_flags,
            texture_type,
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
            wgpu::TextureFormat::Rgba8Unorm => 4,
            wgpu::TextureFormat::Bgra8Unorm => 4,
            wgpu::TextureFormat::R8Unorm => 1,
            _ => panic!("Unsupported texture format"),
        } * size.width;

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
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
    pub fn size(&self) -> wgpu::Extent3d {
        self.texture.size()
    }

}

pub struct TextureManager {
    stencil_texture: StencilTexture,
    pub textures: Slab<Texture>,
    pub place_holder_texture: Texture,
    pub layout: wgpu::BindGroupLayout,
}

impl TextureManager {
    pub fn new(device: &wgpu::Device, surface_config: &wgpu::SurfaceConfiguration) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("NVG Texture Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let place_holder_texture = Texture::placeholder_texture(&device, &layout);

        return Self {
            stencil_texture: StencilTexture::new(
                device,
                surface_config.width,
                surface_config.height,
            ),
            textures: Slab::new(),
            place_holder_texture,
            layout,
        };
    }

    #[inline]
    pub fn configure_stencil(
        &mut self,
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
    ) {
        self.stencil_texture =
            StencilTexture::new(device, surface_config.width, surface_config.height);
    }

    #[inline]
    pub fn stencil_view(&self) -> &wgpu::TextureView {
        return &self.stencil_texture.view;
    }

    #[inline]
    pub fn create(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: wgpu::Extent3d,
        flags: ImageFlags,
        texture_type: TextureType,
        data: Option<&[u8]>,
    ) -> usize {
        let texture = Texture::new(&device, size, flags, texture_type, &self.layout);
        if let Some(data) = data {
            texture.update(&queue, data, wgpu::Origin2d::ZERO, size);
        }
        return self.textures.insert(texture);
    }

    #[inline]
    pub fn get(&self, id: usize) -> Option<&Texture> {
        self.textures.get(id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Texture> {
        self.textures.get_mut(id)
    }

    #[inline]
    pub fn remove(&mut self, id: usize) -> Texture {
        self.textures.remove(id)
    }

    #[inline]
    pub fn get_bindgroup(&self, id: Option<usize>) -> &wgpu::BindGroup {
        if let Some(id) = id {
            return &self
                .get(id)
                .unwrap_or(&self.place_holder_texture)
                .bind_group;
        } else {
            return &self.place_holder_texture.bind_group;
        }
    }
}

#[inline]
pub fn texture_type_map(texture_type: nvgx::TextureType) -> wgpu::TextureFormat {
    match texture_type {
        nvgx::TextureType::RGBA => wgpu::TextureFormat::Rgba8Unorm,
        nvgx::TextureType::BGRA => wgpu::TextureFormat::Bgra8Unorm,
        nvgx::TextureType::Alpha => wgpu::TextureFormat::R8Unorm,
    }
}

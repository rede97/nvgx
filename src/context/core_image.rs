use super::{Context, ImageFlags, ImageId, TextureType};
use crate::RendererDevice;

impl<R: RendererDevice> Context<R> {
    pub fn create_image_init<D: AsRef<[u8]>>(
        &mut self,
        flags: ImageFlags,
        data: D,
    ) -> anyhow::Result<ImageId> {
        let img = image::load_from_memory(data.as_ref())?;
        let img = img.to_rgba8();
        let dimensions = img.dimensions();
        let img = self.renderer.create_texture(
            TextureType::RGBA,
            dimensions.0,
            dimensions.1,
            flags,
            Some(&img.into_raw()),
        )?;
        Ok(img)
    }

    pub fn create_image_from_file<P: AsRef<std::path::Path>>(
        &mut self,
        flags: ImageFlags,
        path: P,
    ) -> anyhow::Result<ImageId> {
        self.create_image_init(flags, std::fs::read(path)?)
    }

    pub fn create_image(
        &mut self,
        width: u32,
        height: u32,
        fmt: TextureType,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId> {
        let img = self
            .renderer
            .create_texture(fmt, width, height, flags, data)?;
        Ok(img)
    }

    /// area: Some(x, y, w, h)
    pub fn update_image(
        &mut self,
        img: ImageId,
        data: &[u8],
        area: Option<(u32, u32, u32, u32)>,
    ) -> anyhow::Result<()> {
        let (x, y, w, h) = if let Some(area) = area {
            area
        } else {
            let (w, h) = self.image_size(img.clone())?;
            (0, 0, w, h)
        };
        self.renderer.update_texture(img, x, y, w, h, data)?;
        Ok(())
    }

    pub fn image_size(&self, img: ImageId) -> anyhow::Result<(u32, u32)> {
        let res = self.renderer.texture_size(img)?;
        Ok(res)
    }

    pub fn delete_image(&mut self, img: ImageId) -> anyhow::Result<()> {
        self.renderer.delete_texture(img)?;
        Ok(())
    }
}

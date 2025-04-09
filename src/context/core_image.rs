use super::{Context, ImageFlags, ImageId, TextureType};
use crate::Renderer;

impl<R: Renderer> Context<R> {
    pub fn create_image<D: AsRef<[u8]>>(
        &mut self,
        flags: ImageFlags,
        data: D,
    ) -> anyhow::Result<ImageId> {
        let img = image::load_from_memory(data.as_ref())?;
        let img = img.to_rgba();
        let dimensions = img.dimensions();
        let img = self.renderer.create_texture(
            TextureType::RGBA,
            dimensions.0 as usize,
            dimensions.1 as usize,
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
        self.create_image(flags, std::fs::read(path)?)
    }

    pub fn create_image_rgba(
        &mut self,
        width: usize,
        height: usize,
        flags: ImageFlags,
        data: Option<&[u8]>,
    ) -> anyhow::Result<ImageId> {
        let img = self
            .renderer
            .create_texture(TextureType::RGBA, width, height, flags, data)?;
        Ok(img)
    }

    pub fn update_image(&mut self, img: ImageId, data: &[u8]) -> anyhow::Result<()> {
        let (w, h) = self.renderer.texture_size(img.clone())?;
        self.renderer.update_texture(img, 0, 0, w, h, data)?;
        Ok(())
    }

    pub fn image_size(&self, img: ImageId) -> anyhow::Result<(usize, usize)> {
        let res = self.renderer.texture_size(img)?;
        Ok(res)
    }

    pub fn delete_image(&mut self, img: ImageId) -> anyhow::Result<()> {
        self.renderer.delete_texture(img)?;
        Ok(())
    }
}

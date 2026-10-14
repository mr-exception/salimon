//! Explicit native evidence capture, copied before presentation. Ordinary frames never read back.
use crate::RendererError;
use std::time::Duration;
pub(crate) struct Readback {
    buffer: wgpu::Buffer,
    width: u32,
    height: u32,
    stride: u32,
    bgra: bool,
}
impl Readback {
    pub(crate) fn encode(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
    ) -> Result<Self, RendererError> {
        let bgra = match texture.format() {
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => true,
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => false,
            _ => {
                return Err(RendererError::new(
                    "frame capture failed",
                    "capture requires an RGBA8/BGRA8 surface",
                ));
            }
        };
        let size = texture.size();
        let stride = (size.width * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("explicit frame capture"),
            size: u64::from(stride) * u64::from(size.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(stride),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );
        Ok(Self {
            buffer,
            width: size.width,
            height: size.height,
            stride,
            bgra,
        })
    }
    pub(crate) fn finish(self, device: &wgpu::Device) -> Result<Vec<u8>, RendererError> {
        let fail = |e: String| RendererError::new("frame capture failed", e);
        let (send, receive) = std::sync::mpsc::channel();
        self.buffer
            .map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = send.send(result);
            });
        device
            .poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(2)),
            })
            .map_err(|e| fail(e.to_string()))?;
        receive
            .recv_timeout(Duration::from_secs(2))
            .map_err(|e| fail(e.to_string()))?
            .map_err(|e| fail(e.to_string()))?;
        let mapped = self
            .buffer
            .get_mapped_range(..)
            .map_err(|e| fail(e.to_string()))?;
        let mut rgba = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in mapped.chunks_exact(self.stride as usize) {
            for pixel in row[..self.width as usize * 4].chunks_exact(4) {
                rgba.extend(if self.bgra {
                    [pixel[2], pixel[1], pixel[0], pixel[3]]
                } else {
                    [pixel[0], pixel[1], pixel[2], pixel[3]]
                });
            }
        }
        drop(mapped);
        self.buffer.unmap();
        let mut output = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut output, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().map_err(|e| fail(e.to_string()))?;
            writer
                .write_image_data(&rgba)
                .map_err(|e| fail(e.to_string()))?;
        }
        Ok(output)
    }
}

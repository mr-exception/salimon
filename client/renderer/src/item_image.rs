//! Bounded PNG decoding for the authored handheld atlas and derived toolbar art.
use crate::RendererError;
use std::io::Cursor;

pub(crate) struct ItemImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub(crate) fn decode(bytes: &[u8]) -> Result<ItemImage, RendererError> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(error)?;
    let info = reader.info();
    if info.width == 0 || info.height == 0 || info.width > 256 || info.height > 256 {
        return Err(error("image must fit 256x256"));
    }
    let mut buffer = vec![
        0;
        reader
            .output_buffer_size()
            .ok_or_else(|| error("image size overflow"))?
    ];
    let info = reader.next_frame(&mut buffer).map_err(error)?;
    let source = &buffer[..info.buffer_size()];
    let pixels = match info.color_type {
        png::ColorType::Rgba => source.to_vec(),
        png::ColorType::Rgb => source
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        _ => return Err(error("expected RGB/RGBA image")),
    };
    Ok(ItemImage {
        width: info.width,
        height: info.height,
        pixels,
    })
}

fn error(e: impl std::fmt::Display) -> RendererError {
    RendererError::new("decode item.mining-tool PNG", e)
}

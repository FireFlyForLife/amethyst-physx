use amethyst::renderer::palette::Srgba;

pub fn unpack_color(packed: u32) -> Srgba {
    let r: u32 = (packed) & 0xFF;
    let g: u32 = (packed >> 8) & 0xFF;
    let b: u32 = (packed >> 16) & 0xFF;
    let a: u32 = (packed >> 24) & 0xFF;

    Srgba::new((r as f32) / 255.0, (g as f32) / 255.0, (b as f32) / 255.0, (a as f32) / 255.0 )
}

// pub fn pack_color(unpacked: Srgba) -> u32{

// }

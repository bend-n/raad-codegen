#[derive(raad_codegen::Write, raad_codegen::Read)]
struct Header<T, U> {
    #[raad(equals)]
    magic: [u8; 4],
    width: u32,
    height: u32,
    channels: u8,
    colorspace: u8,
    yar: T,
    var: U,
}

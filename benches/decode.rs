use criterion::{criterion_group, criterion_main, Criterion};
use embedded_graphics::framebuffer::Framebuffer;
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::ImageDrawable;

fn decode_gif() {
    let im = tinygif::Gif::from_slice(include_bytes!("../assets/Ferris-240x240.gif")).unwrap();

    let mut fb: Framebuffer<
        Rgb565,
        _,
        LittleEndian,
        240,
        240,
        { embedded_graphics::framebuffer::buffer_size::<Rgb565>(240, 240) },
    > = Framebuffer::new();

    for frame in im.frames() {
        frame.draw(&mut fb).unwrap(); // dummy draw and color mapping from Rgb888 to Rgb565
    }
}

fn bench_gif_decoder(c: &mut Criterion) {
    c.bench_function("decode Animated Ferris", |b| {
        b.iter(|| decode_gif())
    });
}

criterion_group!(benches, bench_gif_decoder);
criterion_main!(benches);

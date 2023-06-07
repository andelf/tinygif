# tinygif

A tiny gif decoder written in `no_std` Rust.
This crate requires about 20kB of memory to decode a gif.

- [x] basic decoding
- [x] frame iterator
- [ ] interlace support

## Usage

```rust
let image = tinygif::Gif::<Rgb565>::from_slice(include_bytes!("../small.gif")).unwrap();
loop {
    for frame in image.frames() {
        info!("frame {:?}", frame);

        frame.draw(&mut display).unwrap();

        let delay_ms = frame.delay_centis * 10;
        info!("delay {}", delay_ms);
        // Delay here
        // Timer::after(Duration::from_millis(delay_ms as u64)).await;


        // Or, draw at given offset
        // (use embedded_graphics::prelude::DrawTargetExt;)
        frame.draw(&mut display.translated(Point::new(30, 50))).unwrap();
    }
}
```

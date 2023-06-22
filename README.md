# tinygif

A tiny gif decoder written in `no_std` Rust.
This crate requires about 20kB of memory to decode a gif.

- [x] basic decoding
- [x] frame iterator
- [ ] interlace support
- [ ] fails on some highly compressed gifs: **Change table size in DecodingDict**

## Usage

```rust
let image = tinygif::Gif::<Rgb565>::from_slice(include_bytes!("../Ferris-240x240.gif")).unwrap();
loop {
    for frame in image.frames() {
        info!("frame {:?}", frame);

        frame.draw(&mut display).unwrap();

        let delay_ms = frame.delay_centis * 10;
        info!("delay {}", delay_ms);
        // Delay here
        // Timer::after(Duration::from_millis(delay_ms as u64)).await;

        // Or, draw at given offset
        // use embedded_graphics::prelude::DrawTargetExt;
        // frame.draw(&mut display.translated(Point::new(30, 50))).unwrap();
    }
}
```

## License

MIT or Apache-2.0 at your option.

### License of the gif files used in test

> Animated Ferris in Action
> Happy as a Rustacean at Rust Fest Berlin 2016 (www.rustfest.eu)

- CC BY 4.0 [Animated Ferris for Rust Fest Berlin by A. L. Palmer](https://www.behance.net/gallery/42774743/Rustacean)
- Resized by [ezgif](https://ezgif.com/resize)

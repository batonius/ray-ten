## ray ten

![Preview](ray-ten.mp4)

A reimagining of a ZX Spectrum game [room ten](https://www.mobygames.com/game/zx-spectrum/room-ten) made with real-time ray-tracing. Copmplitely ignoring GPUs when rendering 3D graphics is extremely inefficient, but at least we are quite efficient at being inefficient, thanks to [portable-simd](https://doc.rust-lang.org/std/simd/index.html) and [rayon](https://docs.rs/rayon/latest/rayon/).

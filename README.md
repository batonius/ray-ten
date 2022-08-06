## ray ten

https://user-images.githubusercontent.com/153945/183267977-6cbcbf57-995d-468a-b778-612700fa3d7c.mp4

A reimagining of a ZX Spectrum game [room ten](https://www.mobygames.com/game/zx-spectrum/room-ten) made with real-time ray-tracing. Copmplitely ignoring GPUs when rendering 3D graphics is extremely inefficient, but ray-ten is quite efficient at being inefficient, thanks to [portable-simd](https://doc.rust-lang.org/std/simd/index.html), [rayon](https://docs.rs/rayon/latest/rayon/) and cutting all the corners.

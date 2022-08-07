## ray ten

[Run in browser](https://batonius.github.io/ray-ten/)

https://user-images.githubusercontent.com/153945/183267977-6cbcbf57-995d-468a-b778-612700fa3d7c.mp4

Reimagining of a 1986 ZX Spectrum game [room ten](https://www.mobygames.com/game/zx-spectrum/room-ten) made with ray tracing.
Completely ignoring GPUs when rendering 3D graphics is extremely inefficient, but ray-ten is quite efficient at being inefficient, thanks to [portable-simd](https://doc.rust-lang.org/std/simd/index.html), [rayon](https://docs.rs/rayon/latest/rayon/) and cutting all the corners.
I use [macroquad](https://github.com/not-fl3/macroquad) for displaying the render and window controls because its WASM integration just works.

**So what do I do?**

Use the arrow keys to move the paddle trying not to miss the ball. 

**I'm on a phone and I have no physical keyboard.**

You can also click/touch borders to move the paddle.

**Why ray tracing tho?**

Because modern graphics pipelines are boring and reflections are nice, even in low-res.

**Couldn't you just use fragment shaders to the same effect? That would be infinitely more efficient.**

That's true, I believe I could, maybe I'll try it next.

**What's 'rps'?**

That's rays-per-second, a handy way to gauge ray tracing throughput. Works like ersatz CPU benchmark unless fps are capped.

**So what's the results then?**

| Platform                | Native single core | Native multicore | Firefox WASM | Chrome WASM |
| ----------------------- | ------------------ | ---------------- | ------------ | ----------- |
| Ryzen 7 3700X (8 cores) | 39M                | 300M             | 7M           | 17M         |
| MediaTek Dimensity 1200 | -                  | -                | 10M          | 12M         |

All browser versions are the most recent as of August 07, 2022. WASM modules are compiled with `+simd128`, optimized with `wasm-opt -O4` and run in single core mode. Yes, mobile Firefox is faster than the desktop version.

**That multicore difference is huge, can't you use rayon in WASM somehow?**

I can, but that would require GitHub Pages to set COOP/COEP headers, and they aren't there yet, see https://github.com/community/community/discussions/13309.

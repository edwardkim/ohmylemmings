# Independent host PCM oracle

Unmodified Nuked-OPL3 source at commit `765ec962e473aeb767e4cba74ffdc8f588ffbfe8`, version 1.8, LGPL-2.1-or-later. Original notices and the complete license are retained. [Pinned source URLs, sizes and SHA-256](../../../assets/manifests/opl_reference.json).

The project compiles these files only into a separate **host verification process**, with `tools/audio/nuked_reference.c`. No C source, FFI, build script or binary enters the Rust/native/WASM application dependency graph. The integer Rust port is independently compared with this implementation, sample for sample, using synthetic data in offline CI and optionally verified original data under ignored `assets/local`.

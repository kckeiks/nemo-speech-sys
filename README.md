# nemo-speech-sys

Raw Rust bindings to NVIDIA's [NeMo-Speech.cpp](https://github.com/NVIDIA/NeMo-Speech.cpp) ASR C API.

## Dependencies

- **Build:** Rust 1.88 or newer. The native SDK and CUDA toolkit are not needed to compile this crate.
- **Run inference:** a separately installed NeMo-Speech.cpp SDK and compatible model.
- **GPU inference:** an NVIDIA driver and the CUDA runtime libraries required by the installed SDK.
- **Run host tests:** a C11 compiler; no SDK, model or GPU required.
- **Regenerate bindings:** bindgen-cli 0.72.1, libclang and Python 3.

The supported SDK revision is [4c101bc](https://github.com/NVIDIA/NeMo-Speech.cpp/tree/4c101bc7113f49101a3e11d2c994c519f41939f6).
Linux x86-64 is validated. Cargo does not install the SDK or download models.

[Apache-2.0 license](https://github.com/kckeiks/nemo-speech-sys/blob/main/LICENSE)

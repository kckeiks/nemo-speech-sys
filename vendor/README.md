# Header provenance

`asr.h`, `LICENSE`, and `NOTICE` are unmodified copies from
[NVIDIA/NeMo-Speech.cpp](https://github.com/NVIDIA/NeMo-Speech.cpp/tree/4c101bc7113f49101a3e11d2c994c519f41939f6),
commit `4c101bc7113f49101a3e11d2c994c519f41939f6`:

- `include/nemo_speech/asr.h`
- `LICENSE`
- `NOTICE`

Only the ASR public header is vendored. No third-party runtime implementations,
SDK binaries, or model weights are redistributed. The header carries NVIDIA's
copyright and Apache-2.0 license identifier; its license and notice accompany it.

When updating, copy the header from a reviewed immutable revision, update
`UPSTREAM_REVISION`, regenerate bindings, and run the C ABI conformance tests.
Audit changes in ownership, threading, default values, and symbol availability
before declaring compatibility. Native libraries must use ordinary C enum
layout (do not build with `-fshort-enums`).

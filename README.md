# nemo-speech-sys

Raw, dynamically loaded bindings to NVIDIA's official NeMo-Speech.cpp ASR C ABI.
This is separate from the community nemotron-asr.cpp runtime.

Normal Cargo builds use checked-in bindings and require neither the native SDK
nor CUDA, Clang, bindgen, or network downloads. At runtime provide an absolute
path to an installed compatible ASR C ABI library (`libnemo_speech_asr_c.so.1` on
Linux). Its native dependencies must also be installed. The loader resolves all
ASR symbols eagerly and retains the library until the `Api` is dropped.

Install the native SDK separately using the pinned upstream
[build guide](https://github.com/NVIDIA/NeMo-Speech.cpp/blob/4c101bc7113f49101a3e11d2c994c519f41939f6/docs/build.md)
or the companion [nemo-speech README](https://github.com/kckeiks/nemo-speech). Cargo does not build or download it. Choose a persistent installation prefix and keep its
libraries together; `/tmp` is only used in the validation records, not required
by these bindings.

```rust,no_run
use nemo_speech_sys::Api;

// SAFETY: the operator has installed and verified a trusted SDK built at the
// pinned UPSTREAM_REVISION, matching the vendored header's ABI and contracts.
let api = unsafe { Api::load("/opt/nemo-speech/lib/libnemo_speech_asr_c.so.1") }?;
// SAFETY: this entry point has no arguments or handle lifetime requirements.
let options = unsafe { (api.nemo_speech_asr_recognition_options_default)() };
assert_eq!(options.size, std::mem::size_of_val(&options));
# Ok::<(), Box<dyn std::error::Error>>(())
```

This is an unsafe raw layer. An absolute path and presence of symbols do not
prove ABI compatibility. Never call a copied function pointer after dropping
the `Api`; never destroy the library before its native handles. Refer to the
vendored header and upstream SDK documentation for all ownership and threading
contracts. Prefer the `nemo-speech` safe wrapper in application code.

Enums are integer aliases plus upstream-named constants, so unknown future
status values do not become invalid Rust enum discriminants. `Default` gives
zeroed POD storage; it does **not** set `size` or choose documented sentinel
defaults. Use the native recognition-options default function where available.

## Regeneration and verification

Run these commands from this package's directory. Install bindgen-cli 0.72.1
separately and provide libclang and Python 3 only when regenerating bindings:

```sh
cargo install bindgen-cli --version 0.72.1 --locked
bash tools/regenerate.sh
cargo test -p nemo-speech-sys
```

To validate the checked-in bindings without regenerating them:

```sh
cargo test -p nemo-speech-sys
cargo clippy -p nemo-speech-sys --all-targets -- -D warnings
cargo fmt -p nemo-speech-sys -- --check
```

The companion `nemo-speech` package documents SDK installation and opt-in
inference checks. `ASR_HEADER` exposes the exact vendored header text so
downstream C fixtures can use the header from their resolved dependency without
assuming a sibling checkout directory.

`BINDGEN=/path/to/bindgen` selects a custom CLI location. Generated code should
be reviewed and committed; no build script silently regenerates it. Regeneration
uses standard C declarations with no packed layout or shortened enums. The
script applies one documented, exact-match normalization to bindgen's dynamic
loader template for libloading 0.9 (`path` becomes `path.as_ref()`). All type and
function declarations remain bindgen-generated.
The checked-in output is platform-neutral for this header's types; the ABI test
checks its actual target layout against a C11 compiler on the current host.

Tests require a C11 compiler (`CC`, default `cc`), but no native SDK, model, or GPU.
The conformance test checks the size/alignment of all public concrete structs,
every field offset, and both enum sizes and values against the vendored header.
It supports Unix host tests; Windows compilation/loading has not been validated.

## Standalone repository

This directory is an independent Cargo package. All source, ABI fixtures,
regeneration tools, and upstream header/license notices are included.

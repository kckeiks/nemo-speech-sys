#!/usr/bin/env bash
set -euo pipefail

# Install separately: cargo install bindgen-cli --version 0.72.1 --locked
# BINDGEN may point to an alternate installation. No download is performed here.
crate_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
bindgen_bin="${BINDGEN:-bindgen}"
if [[ "$("$bindgen_bin" --version)" != "bindgen 0.72.1" ]]; then
    echo "Regeneration requires bindgen-cli 0.72.1" >&2
    exit 1
fi

"$bindgen_bin" "$crate_dir/vendor/asr.h" \
    --allowlist-type 'nemo_speech_asr_.*' \
    --allowlist-function 'nemo_speech_asr_.*' \
    --default-enum-style consts \
    --no-prepend-enum-name \
    --with-derive-default \
    --dynamic-loading NativeApi \
    --dynamic-link-require-all \
    --no-layout-tests \
    --wrap-unsafe-ops \
    --rust-edition 2021 \
    --rust-target 1.88 \
    --raw-line '// SPDX-FileCopyrightText: Copyright (c) 2026 NVIDIA CORPORATION & AFFILIATES. All rights reserved.' \
    --raw-line '// SPDX-License-Identifier: Apache-2.0' \
    --raw-line '// Generated declarations from vendor/asr.h; loader normalized for libloading 0.9 by tools/regenerate.sh.' \
    --output "$crate_dir/src/bindings.rs" \
    -- -x c -std=c11

# bindgen 0.72.1's dynamic-loader template targets libloading 0.8's generic
# AsRef<OsStr> input. 0.9 uses AsFilename, implemented by &OsStr; preserve the
# generated API and signatures and normalize only the constructor argument.
python3 - "$crate_dir/src/bindings.rs" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text()
old = "::libloading::Library::new(path)"
assert text.count(old) == 1, "bindgen loader template changed; review required"
path.write_text(text.replace(old, "::libloading::Library::new(path.as_ref())"))
PY

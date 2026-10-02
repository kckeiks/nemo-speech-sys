//! Raw bindings to the pinned NVIDIA NeMo-Speech.cpp ASR C ABI.
//!
//! No SDK, CUDA, C compiler, or network access is required to build this crate.
//! [`Api::load`] loads a trusted installed library at runtime. Every ASR symbol
//! in the vendored header is required, so missing symbols fail during loading.
//!
//! The caller must obey the C header's ownership and threading requirements.
//! Keep the [`Api`] alive until every native handle and copied function pointer
//! obtained from it is no longer used. A recognizer must outlive its streams;
//! result strings are borrowed until result destruction. Drive each stream
//! from a single thread. Copy thread-local errors before calling another API.
//!
//! C enums are represented as integers so future status values remain valid
//! Rust values. Config structs must set `size` to their Rust `size_of` and must
//! use the documented sentinel values; zeroing every config is not equivalent
//! to selecting the library defaults.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
#[allow(clippy::missing_safety_doc, clippy::too_many_arguments)]
mod bindings;

pub use bindings::NativeApi as Api;
pub use bindings::*;

use std::path::{Path, PathBuf};

/// NVIDIA/NeMo-Speech.cpp commit used for the vendored ASR header.
pub const UPSTREAM_REVISION: &str = "4c101bc7113f49101a3e11d2c994c519f41939f6";

/// Exact pinned ASR header, available to downstream C ABI conformance fixtures.
///
/// This is header text only; no native SDK binaries are embedded.
pub const ASR_HEADER: &str = include_str!("../vendor/asr.h");

/// Failure to select or load a native SDK library.
#[derive(Debug)]
pub enum LoadError {
    /// An absolute path is required to avoid platform library search rules.
    RelativePath(PathBuf),
    /// The selected library or a required symbol could not be loaded.
    Library(libloading::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RelativePath(path) => write!(
                f,
                "an absolute NeMo library path is required: {}",
                path.display()
            ),
            Self::Library(error) => write!(f, "failed to load the NeMo ASR library: {error}"),
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::RelativePath(_) => None,
            Self::Library(error) => Some(error),
        }
    }
}

impl Api {
    /// Load all ASR entry points from an explicitly selected native library.
    ///
    /// The library is retained by this object. Its runtime version string is
    /// informational; it cannot prove binary compatibility. The generated
    /// `new` and `from_library` constructors are lower-level escape hatches.
    /// Prefer this constructor, which requires an absolute path.
    ///
    /// # Safety
    ///
    /// The selected library and its dependencies must be trusted: loading can
    /// execute initialization and termination code. Every exported ASR symbol
    /// must implement the ABI and ownership/threading contracts of the
    /// vendored header, including standard C enum size and struct layouts.
    /// Use the pinned upstream revision, or independently validate ABI
    /// compatibility. Keep this object alive while its functions or native
    /// handles are in use. The loader checks symbol presence, not signatures.
    pub unsafe fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let path = path.as_ref();
        if !path.is_absolute() {
            return Err(LoadError::RelativePath(path.to_owned()));
        }
        // SAFETY: The caller guarantees the selected library's compatibility
        // and trust; bindgen retains its Library and requires every symbol.
        unsafe { Self::new(path) }.map_err(LoadError::Library)
    }
}

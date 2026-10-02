//! Compare Rust layout against a host C compiler, without linking the SDK.
#![cfg(unix)]

use nemo_speech_sys::*;
use std::{collections::BTreeMap, fs, path::PathBuf, process::Command};

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let dir = Self(std::env::temp_dir().join(format!(
            "nemo-abi-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        )));
        fs::create_dir(&dir.0).unwrap();
        dir
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn rust_layout_matches_the_vendored_c_header() {
    let dir = TempDir::new();
    let mut source = String::from("#include <stdio.h>\n#include \"asr.h\"\nint main(void) {\n");
    let mut rust = BTreeMap::new();

    macro_rules! layout {
        ($ty:ident, $($field:ident),+ $(,)?) => {{
            let name = stringify!($ty);
            source.push_str(&format!("printf(\"{name}.size %zu\\n\", sizeof({name}));\n"));
            source.push_str(&format!("printf(\"{name}.align %zu\\n\", _Alignof({name}));\n"));
            rust.insert(format!("{name}.size"), std::mem::size_of::<$ty>());
            rust.insert(format!("{name}.align"), std::mem::align_of::<$ty>());
            $(
                let field = stringify!($field);
                source.push_str(&format!("printf(\"{name}.{field} %zu\\n\", offsetof({name}, {field}));\n"));
                rust.insert(format!("{name}.{field}"), std::mem::offset_of!($ty, $field));
            )+
        }};
    }

    layout!(nemo_speech_asr_backend_config, size, gpu);
    layout!(nemo_speech_asr_model_config, size, path, name);
    layout!(
        nemo_speech_asr_streaming_config,
        size,
        chunk_size,
        ctc_left_padding,
        ctc_right_padding,
        rnnt_right_context
    );
    layout!(
        nemo_speech_asr_decoder_config,
        size,
        kind,
        flashlight_lm,
        flashlight_lexicon,
        flashlight_tokenizer,
        beam_size,
        beam_size_token,
        beam_threshold,
        lm_weight,
        word_insertion_score,
        max_boost
    );
    layout!(
        nemo_speech_asr_vad_config,
        size,
        model_path,
        enable_masking,
        onset,
        offset
    );
    layout!(
        nemo_speech_asr_endpointing_config,
        size,
        enable,
        vad_based,
        stop_history_eou_ms
    );
    layout!(
        nemo_speech_asr_postproc_config,
        size,
        profanity_list_path,
        itn_model_dir,
        pnc_model_path
    );
    layout!(
        nemo_speech_asr_diar_config,
        size,
        model_path,
        chunk_frames,
        right_context_frames,
        left_context_frames,
        fifo_frames,
        spkcache_frames,
        update_period_frames
    );
    layout!(
        nemo_speech_asr_batching_config,
        size,
        enable,
        max_batch_size,
        max_queue_delay_us,
        max_queue_depth,
        ingress_cohort_delay_us,
        state_arena_slots
    );
    layout!(
        nemo_speech_asr_recognizer_config,
        size,
        backend,
        model,
        streaming,
        decoder,
        vad,
        endpointing,
        postproc,
        diar,
        batching
    );
    layout!(
        nemo_speech_asr_speech_context,
        size,
        phrases,
        phrase_count,
        boost
    );
    layout!(
        nemo_speech_asr_recognition_options,
        size,
        request_id,
        language_code,
        interim_results,
        enable_word_time_offsets,
        enable_automatic_punctuation,
        verbatim_transcripts,
        profanity_filter,
        stop_history_eou_ms,
        speech_contexts,
        speech_context_count,
        max_alternatives,
        enable_speaker_diarization,
        max_speaker_count
    );

    macro_rules! scalar {
        ($ty:ty) => {{
            let name = stringify!($ty);
            source.push_str(&format!(
                "printf(\"{name}.size %zu\\n\", sizeof({name}));\n"
            ));
            source.push_str(&format!(
                "printf(\"{name}.align %zu\\n\", _Alignof({name}));\n"
            ));
            rust.insert(format!("{name}.size"), std::mem::size_of::<$ty>());
            rust.insert(format!("{name}.align"), std::mem::align_of::<$ty>());
        }};
    }
    scalar!(nemo_speech_asr_status);
    scalar!(nemo_speech_asr_decoder_kind);
    macro_rules! value {
        ($name:ident) => {{
            let name = stringify!($name);
            source.push_str(&format!("printf(\"{name} %u\\n\", (unsigned){name});\n"));
            rust.insert(name.to_string(), $name as usize);
        }};
    }
    value!(NEMO_SPEECH_ASR_OK);
    value!(NEMO_SPEECH_ASR_ERROR_INVALID_ARGUMENT);
    value!(NEMO_SPEECH_ASR_ERROR_OUT_OF_MEMORY);
    value!(NEMO_SPEECH_ASR_ERROR_RUNTIME);
    value!(NEMO_SPEECH_ASR_ERROR_CANCELLED);
    value!(NEMO_SPEECH_ASR_DECODER_GREEDY);
    value!(NEMO_SPEECH_ASR_DECODER_FLASHLIGHT);
    source.push_str("return 0;\n}\n");
    fs::write(dir.0.join("layout.c"), source).unwrap();
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let compilation = Command::new(compiler)
        .args(["-std=c11", "-Wall", "-Wextra", "-Werror", "-I"])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor"))
        .arg(dir.0.join("layout.c"))
        .arg("-o")
        .arg(dir.0.join("layout"))
        .output()
        .expect("ABI tests require a C11 compiler (set CC)");
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    let output = Command::new(dir.0.join("layout")).output().unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let c: BTreeMap<String, usize> = text
        .lines()
        .map(|line| {
            let (key, value) = line.split_once(' ').unwrap();
            (key.to_owned(), value.parse().unwrap())
        })
        .collect();
    assert_eq!(rust, c, "Rust and C ASR layouts must agree exactly");
}

#[test]
fn relative_paths_are_rejected_before_loading_code() {
    // SAFETY: the constructor returns before library loading for this path.
    let error = match unsafe { Api::load("libnemo_speech_asr.so") } {
        Err(error) => error,
        Ok(_) => panic!("relative library paths must be rejected"),
    };
    assert!(matches!(error, LoadError::RelativePath(_)));
}

#[test]
fn incomplete_libraries_fail_during_loading() {
    let dir = TempDir::new();
    fs::write(
        dir.0.join("incomplete.c"),
        "#include \"asr.h\"\n\
         nemo_speech_asr_recognition_options nemo_speech_asr_recognition_options_default(void) {\n\
             nemo_speech_asr_recognition_options value = {0};\n\
             value.size = sizeof(value); return value;\n\
         }\n",
    )
    .unwrap();
    let library_path = dir.0.join("libincomplete.so");
    let compiler = std::env::var_os("CC").unwrap_or_else(|| "cc".into());
    let compilation = Command::new(compiler)
        .args([
            "-std=c11", "-shared", "-fPIC", "-Wall", "-Wextra", "-Werror", "-I",
        ])
        .arg(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("vendor"))
        .arg(dir.0.join("incomplete.c"))
        .arg("-o")
        .arg(&library_path)
        .output()
        .expect("loader tests require a C11 compiler (set CC)");
    assert!(
        compilation.status.success(),
        "{}",
        String::from_utf8_lossy(&compilation.stderr)
    );
    // SAFETY: this test builds the library from the vendored declaration; its
    // sole export has the correct ABI and no initialization/termination code.
    // All required symbols are resolved before Api can become usable.
    let error = match unsafe { Api::load(&library_path) } {
        Err(error) => error,
        Ok(_) => panic!("an incomplete native library must be rejected"),
    };
    assert!(matches!(error, LoadError::Library(_)));
}

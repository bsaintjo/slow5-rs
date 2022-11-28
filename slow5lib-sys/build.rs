use std::{env, error::Error, path::PathBuf};

use dunce::realpath;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");

    let streamvbyte = realpath("slow5lib/thirdparty/streamvbyte/include")?;
    let library_path = realpath("slow5lib/include")?;
    let other_includes = realpath("slow5lib/src")?;
    let slow5_include = realpath("slow5lib/include/slow5")?;
    let klib_include = realpath("slow5lib/src/klib")?;

    let mut cfg = cc::Build::new();

    let mut includes = vec![
        library_path,
        streamvbyte,
        other_includes,
        slow5_include,
        klib_include,
    ];
    if let Some(include) = std::env::var_os("DEP_Z_INCLUDE") {
        includes.push(PathBuf::from(include));
    }

    if let Some(include) = std::env::var_os("DEP_ZSTD_INCLUDE") {
        cfg.include(include);
        cfg.define("SLOW5_USE_ZSTD", "1");
    } else {
        cfg.define("SLOW5_USE_ZSTD", None);
    }

    println!(
        "cargo:include={}",
        env::join_paths(&includes)?.to_string_lossy()
    );
    println!("cargo:root={}", env::var("OUT_DIR")?);

    for path in includes {
        cfg.include(path);
    }

    cfg.file("slow5lib/thirdparty/streamvbyte/src/streamvbyte_encode.c")
        .file("slow5lib/thirdparty/streamvbyte/src/streamvbyte_decode.c")
        .file("slow5lib/thirdparty/streamvbyte/src/streamvbyte_zigzag.c")
        .file("slow5lib/src/slow5.c")
        .file("slow5lib/src/slow5_idx.c")
        .file("slow5lib/src/slow5_misc.c")
        .file("slow5lib/src/slow5_press.c");

    cfg.shared_flag(false)
        .flag("-std=c99")
        // TODO Check if I need a target dependent cfg to support these
        // .flag("-mssse3")
        // .define("STREAMVBYTE_SSSE3", "1")
        // .define("__ARM_NEON__", "1")
        .opt_level(3)
        .compile("slow5");

    let bindings = bindgen::Builder::default()
        .header("slow5lib/include/slow5/slow5.h")
        .header("slow5lib/include/slow5/klib/khash.h")
        .header("slow5lib/include/slow5/klib/kvec.h")
        .header("slow5lib/include/slow5/slow5_defs.h")
        .header("slow5lib/include/slow5/slow5_error.h")
        .header("slow5lib/include/slow5/slow5_press.h")
        .header("slow5lib/src/slow5_misc.h")
        .header("slow5lib/src/slow5_idx.h")
        .header("slow5lib/src/slow5_extra.h")
        .header("slow5lib/src/klib/ksort.h")
        .clang_arg("-Islow5lib/include")
        // slow5*.h
        .allowlist_function("slow5_.*")
        .allowlist_var("SLOW5_.*")
        .allowlist_type("slow5_.*")
        .allowlist_type("__slow5_press")
        .allowlist_type("__va_list_tag")
        .allowlist_type("va_list")
        .allowlist_type("__darwin_va_list")
        .allowlist_type("__builtin_va_list")
        .allowlist_type("key_t")
        .allowlist_type("__int32_t")
        // khash.h
        .allowlist_type("kh.*")
        .allowlist_function("kh.*")
        .allowlist_type("k.*")
        .allowlist_function("k.*")
        .allowlist_type("__k.*")
        // // kvec.h
        .allowlist_type("kv.*")
        .allowlist_function("kv.*")
        // // ksort.h
        // .allowlist_type("ks.*")
        // .allowlist_function("ks.*")
        .size_t_is_usize(true)
        .rustfmt_bindings(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .ctypes_prefix("libc")
        .allowlist_recursively(false)
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings");

    println!("cargo:rustc-link-lib=slow5");
    println!("cargo:rustc-link-lib=zstd");
    println!("cargo:rustc-link-lib=z");
    Ok(())
}

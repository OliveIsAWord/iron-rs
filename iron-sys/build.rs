use std::{env, path::PathBuf};

use bindgen::{AliasVariation, callbacks::ParseCallbacks};
use walkdir::WalkDir;

fn main() {
    let source_files = WalkDir::new("vendor/src/iron")
        .into_iter()
        // The driver directory is for compiling Iron as an executable.
        .filter_entry(|entry| entry.file_name() != "driver")
        .map(|entry| entry.unwrap().path().to_owned())
        .filter(|entry| entry.extension().is_some_and(|ext| ext == "c"))
        .inspect(|e| println!("cargo::rerun-if-changed={}", e.as_os_str().display()));
    let flags = &[
        "-std=gnu2x",
        "-Wno-deprecated-declarations",
        "-Wno-incompatible-pointer-types-discards-qualifiers",
        "-Wno-initializer-overrides",
        "-Wno-unused",
        "-Wno-unused-parameter",
    ];
    let mut build = cc::Build::new();
    build
        .compiler("clang") // uses GCC extensions
        .include("vendor/src")
        .opt_level(0) // i don't trust the sandwich man
        .files(source_files);
    for flag in flags {
        build.flag(flag);
    }
    build.compile("iron");
    let header = "vendor/src/iron/iron.h";
    println!("cargo::rerun-if-changed={header}");
    let bindings = bindgen::Builder::default()
        .header(header)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_arg("-std=c23")
        .allowlist_file(header)
        .use_core()
        .generate_cstr(true)
        .merge_extern_blocks(true)
        .parse_callbacks(Box::new(RemoveFePrefix))
        // stop `u8` from being escaped as `u8_`, etc.
        .clang_args([
            "-Du8=uint8_t",
            "-Du16=uint16_t",
            "-Du32=uint32_t",
            "-Du64=uint64_t",
            "-Dusize=uintptr_t",
            "-Di8=int8_t",
            "-Di16=int16_t",
            "-Di32=int32_t",
            "-Di64=int64_t",
            "-Disize=intptr_t",
        ])
        // stop `f32` from being escaped as `f32_`, etc.
        .blocklist_item("f[0-9]+_?")
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        // These enums are exhaustive.
        .rustified_enum("Fe(SymbolBinding|SymbolKind|RegStatus)")
        .bitfield_enum("FeTrait")
        .prepend_enum_name(false)
        .default_alias_style(AliasVariation::NewType)
        .wrap_unsafe_ops(true)
        .flexarray_dst(true)
        .generate()
        .unwrap();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();
}

#[derive(Debug)]
struct RemoveFePrefix;

impl ParseCallbacks for RemoveFePrefix {
    fn item_name(&self, original_item_name: &str) -> Option<String> {
        let prefixes = ["fe_", "Fe", "FE_"];
        prefixes
            .into_iter()
            .find_map(|prefix| original_item_name.strip_prefix(prefix))
            .map(str::to_owned)
    }
    fn enum_variant_name(
        &self,
        enum_name: Option<&str>,
        mut variant: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        match variant {
            "FE_ARCH_X86_64" => return Some("X86_64".into()),
            // Prior art: rustc spells it `Stdcall`
            // https://github.com/rust-lang/rust/blob/3ef8e64ce9f72ee8d600d55bc43b36eed069b252/compiler/rustc_abi/src/extern_abi.rs#L28
            "FE_CCONV_STDCALL" => {}
            "FE_CCONV_SYSV" => return Some("SysV".into()),
            "FE_SIGNEXT" => return Some("SignExt".into()),
            "FE_ZEROEXT" => return Some("ZeroExt".into()),
            "FE_BITCAST" => return Some("BitCast".into()),
            "FE_I2F" => return Some("I2F".into()),
            "FE_F2I" => return Some("F2I".into()),
            _ => {}
        }
        let enum_name = enum_name?;
        let _ = enum_name.chars().next().filter(char::is_ascii_uppercase)?; // stupid line for stupid problem
        let prefix = match enum_name {
            "FeCallConv" => "FE_CCONV_".to_owned(),
            "FeSymbolBinding" => "FE_BIND_".to_owned(),
            "FeSymbolKind" => "FE_SYMKIND_".to_owned(),
            "FeRegStatus" => "FE_REG_".to_owned(),
            "FeInstKindGeneric" => "FE_".to_owned(),
            _ => {
                let mut prefix = String::new();
                for c in enum_name.chars() {
                    if prefix.is_empty() {
                        assert!(
                            c.is_ascii_uppercase(),
                            "expected uppercase initial letter in enum name {enum_name:?}"
                        );
                        prefix.push(c);
                    } else if c.is_ascii_lowercase() {
                        prefix.push(c.to_ascii_uppercase());
                    } else if c.is_ascii_uppercase() {
                        prefix.push('_');
                        prefix.push(c);
                    } else {
                        panic!("unexpected character {c:?} in enum name {enum_name:?}");
                    }
                }
                prefix.push('_');
                prefix
            }
        };
        let mut stripped_name = String::new();
        if let Some(no_underscore) = variant.strip_prefix('_') {
            variant = no_underscore;
            stripped_name.push('_');
        }
        variant = variant
            .strip_prefix(&prefix)
            .unwrap_or_else(|| panic!("badness {enum_name:?} {prefix:?} {variant:?}"));
        // `Trait::*` are constants, so keep them in SCREAMING_SNAKE_CASE
        if enum_name == "FeTrait" {
            stripped_name.push_str(variant);
        } else {
            // Format `FE_IADD` as `IAdd`, etc.
            if enum_name == "FeInstKindGeneric" && !matches!(variant, "UPSILON") {
                let type_prefix = variant.chars().next().unwrap();
                if matches!(type_prefix, 'I' | 'U' | 'F') {
                    variant = &variant[1..];
                    stripped_name.push(type_prefix);
                }
            }
            // Rewrite SCREAMING_SNAKE_CASE to TitleCase
            let mut capital = true;
            for mut c in variant.chars() {
                if c == '_' {
                    assert!(!capital, "double underscore in enum variant {variant:?}");
                    capital = true;
                } else {
                    if capital {
                        capital = false;
                    } else {
                        c = c.to_ascii_lowercase();
                    }
                    stripped_name.push(c);
                }
            }
        }
        Some(stripped_name)
    }
}

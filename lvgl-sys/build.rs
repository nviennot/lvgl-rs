use cc::Build;
use std::{env, path::Path, path::PathBuf};

static CONFIG_NAME: &str = "DEP_LV_CONFIG_PATH";

fn main() {
    let project_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .canonicalize()
        .unwrap();
    let shims_dir = project_dir.join("shims");
    let vendor = project_dir.join("vendor");
    let vendor_src = vendor.join("lvgl").join("src");

    let lv_config_dir = {
        let conf_path = env::var(CONFIG_NAME)
            .map(|raw_path| PathBuf::from(raw_path))
            .unwrap_or_else(|_| {
                match std::env::var("DOCS_RS") {
                    Ok(_) => {
                        // We've detected that we are building for docs.rs
                        // so let's use the vendored `lv_conf.h` file.
                        vendor.join("include")
                    }
                    Err(_) => panic!(
                        "The environment variable {} is required to be defined",
                        CONFIG_NAME
                    ),
                }
            });

        if !conf_path.exists() {
            panic!(
                "Directory {} referenced by {} needs to exist",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }
        if !conf_path.is_dir() {
            panic!("{} needs to be a directory", CONFIG_NAME);
        }
        if !conf_path.join("lv_conf.h").exists() {
            panic!(
                "Directory {} referenced by {} needs to contain a file called lv_conf.h",
                conf_path.to_string_lossy(),
                CONFIG_NAME
            );
        }

        println!(
            "cargo:rerun-if-changed={}",
            conf_path.join("lv_conf.h").to_str().unwrap()
        );
        conf_path
    };

    let mut cfg = Build::new();
    add_c_files(&mut cfg, &vendor_src);
    add_c_files(&mut cfg, &shims_dir);

    cfg.define("LV_CONF_INCLUDE_SIMPLE", Some("1"))
        .include(&vendor_src)
        .include(&vendor)
        .warnings(false)
        .include(&lv_config_dir)
        .archiver("arm-none-eabi-ar") // To avoid "has no symbols" warnings
        .flag("-fshort-enums")
        .compile("lvgl");

    let mut cc_args = vec![
        "-DLV_CONF_INCLUDE_SIMPLE=1",
        "-I",
        lv_config_dir.to_str().unwrap(),
        "-I",
        vendor.to_str().unwrap(),
        "-fvisibility=default",
        "-fshort-enums",
    ];

    // Set correct target triple for bindgen when cross-compiling
    let target = env::var("TARGET").expect("Cargo build scripts always have TARGET");
    let host = env::var("HOST").expect("Cargo build scripts always have HOST");
    if target != host {
        cc_args.push("-target");
        cc_args.push(target.as_str());
    }

    let mut additional_args = Vec::new();
    if target.ends_with("emscripten") {
        if let Ok(em_path) = env::var("EMSDK") {
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/include/libc",
                em_path
            ));
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/lib/libc/musl/arch/emscripten",
                em_path
            ));
            additional_args.push("-I".to_string());
            additional_args.push(format!(
                "{}/upstream/emscripten/system/include/SDL",
                em_path
            ));
        }
    }

    let include_paths = String::from_utf8(
        Build::new()
            .get_compiler()
            .to_command()
            .arg("-E")
            .arg("-Wp,-v")
            .arg("-xc")
            .arg("/dev/null")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to run the compiler to get paths")
            .wait_with_output()
            .expect("Failed to run the compiler to get paths")
            .stderr,
    )
    .unwrap()
    .lines()
    .filter_map(|line| line.strip_prefix(" "))
    .map(|path| format!("-I{}", path))
    .collect::<Vec<_>>();

    eprintln!("include_paths={:?}", include_paths);

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header(shims_dir.join("lvgl_sys.h").to_str().unwrap())
        //.rustified_enum("lv_indev_state_t")
        .generate_comments(false)
        .derive_default(true)
        .layout_tests(false)
        .use_core()
        .rustfmt_bindings(true)
        .ctypes_prefix("cty")
        .clang_args(&cc_args)
        .clang_args(&additional_args)
        .clang_args(&include_paths)
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Can't write bindings!");
}

fn add_c_files(build: &mut cc::Build, path: impl AsRef<Path>) {
    for entry in glob::glob(path.as_ref().join("**/*.c").to_str().unwrap()).unwrap() {
        let path = entry.unwrap();
        if path.extension().and_then(|s| s.to_str()) == Some("c") {
            build.file(&path);
        }
    }
}

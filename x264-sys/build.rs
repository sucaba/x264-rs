use std::env;
use std::fs::File;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

fn format_write(builder: bindgen::Builder) -> String {
    builder
        .generate()
        .expect("generate failed")
        .to_string()
        .replace("/**", "/*")
        .replace("/*!", "/*")
}

fn main() {
    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is missing"));

    let x264_path = Path::new("../deps/x264");
    let x264_out_path = out_path.join("build");

    build_x264_dependency(&x264_path);

    let pc_file = x264_out_path.join("x264.pc");
    let buildver = pkg_cfg_file_build_ver(&pc_file);

    let mut builder = bindgen::builder()
        .raw_line(format!(
            "pub unsafe fn x264_encoder_open(params: *mut x264_param_t) -> *mut x264_t {{
                               x264_encoder_open_{}(params)
                          }}",
            buildver
        ))
        .header("data/x264.h");

    builder = builder
        .clang_arg("-I")
        .clang_arg(x264_out_path.display().to_string());

    builder = builder.clang_arg("-I").clang_arg("/usr/include");

    // Manually fix the comment so rustdoc won't try to pick them
    let s = format_write(builder);

    let mut file = File::create(out_path.join("x264.rs")).expect("x264.rs is missing");

    let _ = file.write(s.as_bytes());
}

fn pkg_cfg_file_build_ver(file_name: &Path) -> String {
    let file = File::open(file_name).expect(&format!("cannot openfile  '{}'", file_name.display()));
    std::io::BufReader::new(file)
        .lines()
        .find_map(|line| Some(line.ok()?.strip_prefix("Version: ")?.to_string()))
        .expect("no 'Version:' in x264.pc file")
        .split('.')
        .nth(1)
        .expect("Invalid x264.pc file 'Version:' format")
        .to_string()
}

pub fn build_x264_dependency(path: &Path) {
    // Build the project in the path `foo` and installs it in `$OUT_DIR`
    let dst = autotools::Config::new(path)
        .enable_static()
        .disable("cli", None)
        .env("CC", "gcc")
        .make_target("default")
        .build();

    // Simply link the library without using pkg-config
    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-lib=static=x264");
}

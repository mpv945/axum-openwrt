use std::env;
use std::path::PathBuf;

fn main() {
    // 编译 C
    cc::Build::new()
        .file("c_src/foo.c")
        /*.files([
            "c_lib/src/foo.c",
        ])*/
        .include("c_src")
        .flag("-O3")
        .flag("-march=native")   // 性能
        .flag("-fPIC")
        .compile("foo");

    // bindgen 生成 Rust binding
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Ic_src")
        .allowlist_function("foo_.*")
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .unwrap();

    // 告诉 Rust：链接一个名为 libfoo.a（或 foo.lib）的静态库
    // 等价于：
    //    -l:libfoo.a   (Linux)
    //    foo.lib       (Windows)
    println!("cargo:rustc-link-lib=static=foo");
    //如果库不在默认路径; 告诉 linker 去哪里找库
    // println!("cargo:rustc-link-search=native=./lib");

    // Linux/macOS 需要显式链接 math 库
    // Windows 不链接 -lm，MSVC 自带 math ; Linux/macOS 链接 libm
    // 是否需要，问自己一句话： 这个符号是谁实现的？
    if cfg!(target_family = "unix") {
        println!("cargo:rustc-link-lib=m");
    }
}
# 编译生成在： project/target/debug/build/foo-sys-xxxx/out/bindings.rs

很好，这一步就是把你的工程升级成**真正可生产落地的 FFI 构建体系**。我给你一套**完整可用方案**，实现你这三个目标：

> ✅ 提交 `bindings.rs`（CI 不依赖 clang）
> ✅ Linux 使用 musl 生成**完全静态二进制**
> ✅ C 侧可扩展接入第三方库（zlib / OpenSSL 思路）

---

# 一、最终目标架构

```text
ffi-demo/
├── Cargo.toml
├── src/main.rs
├── .github/workflows/ci.yml
├── foo-sys/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── wrapper.h
│   ├── src/
│   │   ├── lib.rs
│   │   └── bindings.rs   ← ✅ 提交到 git（关键）
│   └── c_src/
│       ├── foo.c
│       └── foo.h
└── third_party/
    └── zlib/   ← 示例扩展
```

---

# 二、核心一：固定 bindings.rs（CI 无 clang）

---

## ✅ Step 1：开发机生成

```bash
cargo build
```

找到：

```text
target/debug/build/foo-sys-*/out/bindings.rs
```

复制到：

```text
foo-sys/src/bindings.rs
```

---

## ✅ Step 2：修改 lib.rs

```rust
// foo-sys/src/lib.rs

mod bindings;
pub use bindings::*;

pub fn sqrt(x: f64) -> f64 {
    unsafe { foo_sqrt(x) }
}
```

---

## ❗ Step 3：build.rs 禁用 bindgen

```rust
fn main() {
    let mut build = cc::Build::new();

    build
        .file("c_src/foo.c")
        .include("c_src")
        .flag("-O3");

    if !cfg!(target_env = "msvc") {
        build.flag("-fPIC");
    }

    build.compile("foo");

    println!("cargo:rustc-link-lib=static=foo");

    // Linux/macOS 需要 libm
    if cfg!(target_family = "unix") {
        println!("cargo:rustc-link-lib=m");
    }
}
```

👉 **重点：完全删除 bindgen 代码**

---

# 三、核心二：musl 完全静态二进制（Linux）

---

## ✅ 安装

```bash
rustup target add x86_64-unknown-linux-musl
```

---

## ✅ 构建

```bash
cargo build --release \
  --target x86_64-unknown-linux-musl
```

---

## ✅ 验证

```bash
ldd target/x86_64-unknown-linux-musl/release/ffi-demo
```

输出：

```text
statically linked
```

---

## 🔥 强化（推荐）

```toml
# Cargo.toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

# 四、核心三：GitHub Actions CI（无 clang 版本）

---

## .github/workflows/ci.yml

```yaml
name: Build

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      # Linux: 安装 musl
      - name: Install musl
        if: matrix.os == 'ubuntu-latest'
        run: sudo apt update && sudo apt install -y musl-tools

      - name: Add musl target
        if: matrix.os == 'ubuntu-latest'
        run: rustup target add x86_64-unknown-linux-musl

      # 构建
      - name: Build (Linux musl)
        if: matrix.os == 'ubuntu-latest'
        run: cargo build --release --target x86_64-unknown-linux-musl

      - name: Build (Others)
        if: matrix.os != 'ubuntu-latest'
        run: cargo build --release
```

---

## ✅ CI 特点

| 项目      | 状态    |
| ------- | ----- |
| clang   | ❌ 不需要 |
| bindgen | ❌ 不运行 |
| 速度      | ✅ 更快  |
| 稳定性     | ✅ 更高  |

---

# 五、扩展：接入 zlib（示例）

---

## 目录

```text
third_party/zlib/
```

---

## build.rs 扩展

```rust
// 编译 zlib
cc::Build::new()
    .files([
        "third_party/zlib/adler32.c",
        "third_party/zlib/compress.c",
    ])
    .include("third_party/zlib")
    .compile("z");

// 链接 zlib
println!("cargo:rustc-link-lib=static=z");
```

---

## foo.c 调用 zlib

```c
#include "zlib.h"

int compress_demo() {
    return compressBound(100);
}
```

---

# 六、扩展：OpenSSL（生产建议）

---

## 推荐方案（2025主流）

👉 不手写编译，直接用：

```toml
openssl-sys = { version = "0.9", features = ["vendored"] }
```

👉 或：

```bash
OPENSSL_STATIC=1
```

---

## 原则

| 库类型          | 推荐       |
| ------------ | -------- |
| 小型库          | cc 编译    |
| 大型库（openssl） | vendored |
| 高安全          | 固定版本源码   |

---

# 七、最终生产模式总结

---

## ✅ 开发环境

```text
bindgen 自动生成 bindings.rs
```

---

## ✅ 生产 / CI

```text
使用已提交 bindings.rs
```

---

## ✅ 构建

```text
cc 编译 C → 静态库 → Rust 链接 → 单一二进制
```

---

## ✅ Linux 发布

```text
musl → 完全静态 → 无依赖运行
```

---

# 八、一句话结论

```text
Rust FFI 生产终极形态 =
预生成 bindings + 静态 C + musl + CI 无 clang
```

---

# 九、你现在已经达到的级别

这套方案已经是：

✅ 云原生可部署
✅ 无依赖二进制
✅ 可跨平台 CI
✅ 可扩展 C 生态（zlib/openssl）
✅ ABI 安全

---

# 如果你下一步想再升级（非常推荐）

我可以帮你做：

* 🔥 **cross + 多架构（arm64/x86）发布**
* 🔥 **自动 release 打包（GitHub Release）**
* 🔥 **FFI fuzz + sanitizer（C 安全）**
* 🔥 **完全 hermetic 构建（无系统依赖）**
* 🔥 **Bazel / Nix 构建体系**

直接到**大厂基础设施级别**。

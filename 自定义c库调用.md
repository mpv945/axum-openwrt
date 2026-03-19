完全可以，`foo.c` 里调用系统的 C 库或者你自己编译的 C 项目库是 **FFI 在生产环境中非常常见的做法**。我给你整理一个生产级说明，包含 **静态/动态库、Linux/Windows/macOS** 全套方案和注意事项。

---

# 一、调用系统 C 库

### 示例：调用标准 C 库函数

```c
#include "foo.h"
#include <math.h>
#include <string.h>
#include <stdlib.h>

double foo_sqrt(double x) {
    return sqrt(x);  // 调用系统 math 库
}

void foo_strcpy_demo(char* dst, const char* src) {
    strcpy(dst, src); // 系统 libc
}
```

---

### 编译说明

* Linux/macOS：

    * `libm`、`libc` 是系统库，通常 **自动链接**
    * 如果静态链接，可能需要 `-static` 或 `-lc -lm`
* Windows：

    * MSVC 编译器会自动链接 CRT（C Runtime），通常无需额外操作
    * MinGW 可能需要 `-lmsvcrt` 等

---

# 二、调用自己编译的 C 库（静态或动态）

假设你有一个 C 库 `bar`：

```
third_party/bar/
  ├── include/bar.h
  └── src/bar.c
```

---

### 1️⃣ 静态库

1. 编译生成静态库：

```bash
gcc -O3 -c src/bar.c -I include -o bar.o
ar rcs libbar.a bar.o
```

2. 在 `foo.c` 调用：

```c
#include "bar.h"

int call_bar(int x) {
    return bar_add(x, 10);  // bar 库函数
}
```

3. `build.rs` 链接静态库：

```rust
cc::Build::new()
    .file("c_src/foo.c")
    .include("c_src")
    .include("third_party/bar/include")
    .compile("foo");

// 告诉 Rust 链接静态库 bar
println!("cargo:rustc-link-search=native=third_party/bar");
println!("cargo:rustc-link-lib=static=bar");
```

✅ 优点：无运行时依赖，完全静态

---

### 2️⃣ 动态库（DLL/so/dylib）

1. 编译动态库：

* Linux/macOS：

```bash
gcc -shared -fPIC src/bar.c -I include -o libbar.so
```

* Windows (MinGW)：

```bash
gcc -shared src/bar.c -I include -o bar.dll
```

2. `build.rs` 链接：

```rust
println!("cargo:rustc-link-search=native=third_party/bar");
println!("cargo:rustc-link-lib=dylib=bar"); // 动态库
```

3. 注意：

* Windows 要保证 DLL 在运行时可找到（PATH 或同目录）
* Linux/macOS 要保证 `.so` / `.dylib` 在 `LD_LIBRARY_PATH` 或 `DYLD_LIBRARY_PATH`

✅ 优点：更新库无需重新编译 Rust 二进制
❌ 缺点：增加运行时依赖，可能导致部署复杂

---

# 三、Linux/macOS 静态 vs 动态库的区别

| 类型               | 优点       | 缺点          |
| ---------------- | -------- | ----------- |
| 静态库 (.a)         | 无运行时依赖   | 编译时间长，二进制大  |
| 动态库 (.so/.dylib) | 小二进制，可更新 | 部署复杂，需要环境变量 |

> **生产环境建议**：
>
> * 静态链接核心库（完全控制版本）
> * 动态链接大型第三方库（如 OpenSSL、zlib），部署可控

---

# 四、Windows 特别注意

* MSVC 默认静态 CRT / 动态 CRT 可选（Release 模式建议 `/MT` 静态 CRT）
* 如果调用自己编译的 DLL：

    * Rust 运行时必须能找到 DLL
    * 或者用 `windows-rs` + `link(name = "bar")` 指定

---

# 五、bindgen 绑定规则

* bindgen 生成的 Rust binding 可以直接调用 foo.c 调用的其他库函数，只要它们在 **链接阶段已经链接到二进制**
* Rust 不需要知道 C 内部实现，只要 **头文件声明在 bindgen 里可见**：

```c
// foo.h
int call_bar(int x);   // 内部实现调用 bar_add
```

```rust
// Rust
extern "C" {
    pub fn call_bar(x: i32) -> i32;
}
```

---

# 六、生产级实践总结

1. **核心库（foo-sys / 业务逻辑 C）建议静态编译**
2. **第三方库可选择静态或动态**：

    * 静态：部署简单，无外部依赖
    * 动态：二进制小，可热更新
3. **CI / build.rs 管理库路径**
4. **bindgen 只需要头文件声明**，实际库链接在 build.rs 完成
5. **Windows/Linux/macOS** 都可，注意路径和动态库加载

---

✅ 结论：

> `foo.c` 里完全可以调用 **系统 libc / math** 或者 **自己编译的 C 库**，只要在 **build.rs** 里正确编译 & 链接，并保证 Rust binding 正确声明函数即可。

---

明白，你想要一个**真正跨平台可运行的 demo**，在 Windows、Linux 和 macOS 都可以编译运行，调用一个三个平台都有的系统库进行测试。我们可以用 **标准 C 库 `math` 的 `sqrt`** 作为测试，它在三平台都有。

我给你写一个**完整可运行的 Rust + C FFI demo**，结构清晰，可直接放到 `foo-sys` 下测试。

---

# 一、项目结构

```
ffi-demo/
├── Cargo.toml
├── src/main.rs
├── foo-sys/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── wrapper.h
│   ├── c_src/
│   │   └── foo.c
│   │   └── foo.h
│   └── src/lib.rs
```

---

# 二、C 代码

### foo.h

```c
#pragma once
#include <math.h>  // 系统库

double foo_sqrt(double x);
```

### foo.c

```c
#include "foo.h"

double foo_sqrt(double x) {
    return sqrt(x);  // 调用系统 math 库
}
```

> 注：`math.h` 在 Linux/macOS 是 `libm`，Windows 上是 MSVC 的 CRT math。

---

# 三、wrapper.h（bindgen 使用）

```c
#include "foo.h"
```

---

# 四、foo-sys/build.rs（跨平台写法）

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    let mut build = cc::Build::new();
    build.file("c_src/foo.c").include("c_src").flag("-O3");

    // Windows 不需要 -fPIC
    if !cfg!(target_env = "msvc") {
        build.flag("-fPIC");
    }

    build.compile("foo");

    // bindgen 生成 Rust binding
    if env::var("CI").is_err() {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .clang_arg("-Ic_src")
            .allowlist_function("foo_.*")
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }

    println!("cargo:rustc-link-lib=static=foo");

    // Linux/macOS 需要显式链接 math 库
    if cfg!(target_family = "unix") {
        println!("cargo:rustc-link-lib=m");
    }
}
```

> ✅ 关键点：
>
> * Windows 不链接 `-lm`，MSVC 自带 math
> * Linux/macOS 链接 `libm`
> * build.rs 自动编译 C，生成静态库

---

# 五、foo-sys/src/lib.rs

```rust
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn sqrt(x: f64) -> f64 {
    unsafe { foo_sqrt(x) }
}
```

---

# 六、根项目 src/main.rs

```rust
fn main() {
    let value = 2.0;
    let result = foo_sys::sqrt(value);
    println!("sqrt({}) = {}", value, result);
}
```

---

# 七、编译和运行

### Linux/macOS

```bash
cargo build
cargo run
```

* 输出：

```
sqrt(2) = 1.4142135623730951
```

### Windows (MSVC)

```powershell
cargo build
cargo run
```

* 输出同样正确

---

# 八、说明

1. **跨平台库选择**

    * 我们使用系统 `math` 库，它在 Windows/MSVC、Linux、macOS 都有。
2. **静态 vs 动态**

    * `foo.c` 编译成静态库 `libfoo.a`（Linux/macOS）或 `.lib`（Windows）
    * Rust 静态链接
3. **bindgen**

    * 自动生成 Rust FFI binding，保证 ABI 正确
4. **build.rs**

    * 自动编译 C + 链接系统库 + 生成 binding
    * 避免手动 cargo 配置
5. **CI/生产**

    * 可在 CI 直接 build，Linux/macOS 需要安装 clang/libclang 或提交 bindings.rs
    * Windows 可用 MSVC 编译

---

# 九、验证跨平台

* Linux/macOS:

```bash
ldd target/debug/ffi-demo  # 可以看到链接 libm
```

* Windows:

```powershell
dumpbin /dependents target\debug\ffi-demo.exe
```

✅ 输出表明二进制正常链接

---

# 十、下一步生产优化建议

1. 提交 **bindings.rs** 到 git，CI 不需要 clang
2. 静态编译库 + musl (Linux) 完全无依赖二进制
3. 可以拓展：foo.c 调用自己编译的第三方库，如 zlib、openssl 等

---

我可以帮你写一个**三平台通用 CI 配置**，直接在 GitHub Actions 上编译运行这个 demo，保证 Windows/Linux/macOS 都能过，并生成静态可执行文件。

你希望我写这个吗？


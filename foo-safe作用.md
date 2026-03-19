很好，这个问题非常关键——**`foo-safe` 是 Rust FFI 体系里“生产级设计”的核心层**。

一句话先说清：

> 👉 `foo-sys` = **原始 C 绑定（unsafe）**
> 👉 `foo-safe` = **安全封装（safe Rust API）**

---

# 一、为什么必须有 `foo-safe`

你现在直接这样用：

```rust
unsafe { foo_sqrt(2.0) }
```

问题：

| 风险          | 说明            |
| ----------- | ------------- |
| ❌ 指针错误      | 可能崩溃          |
| ❌ 内存泄漏      | C 分配 Rust 不释放 |
| ❌ 生命周期问题    | Rust 无法检查     |
| ❌ UB（未定义行为） | 生产事故          |

👉 结论：**不能让业务代码直接调用 `foo-sys`**

---

# 二、foo-safe 的职责（本质）

`foo-safe` 做三件事：

---

## 1️⃣ 封装 unsafe

```rust
pub fn sqrt(x: f64) -> f64 {
    unsafe { foo_sys::foo_sqrt(x) }
}
```

👉 把 `unsafe` 隔离在库内部

---

## 2️⃣ 转换类型（C → Rust）

### C：

```c
const char* foo_hello();
```

### foo-safe：

```rust
pub fn hello() -> String {
    unsafe {
        let ptr = foo_sys::foo_hello();
        std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned()
    }
}
```

👉 转换成 Rust 安全类型

---

## 3️⃣ 管理内存（最重要）

### C：

```c
char* foo_alloc_string();
void foo_free(char*);
```

### foo-safe：

```rust
pub fn alloc_string() -> String {
    unsafe {
        let ptr = foo_sys::foo_alloc_string();
        let s = std::ffi::CStr::from_ptr(ptr)
            .to_string_lossy()
            .into_owned();
        foo_sys::foo_free(ptr);
        s
    }
}
```

👉 避免：

* 内存泄漏
* double free
* 野指针

---

# 三、生产级结构（推荐）

```
ffi-demo/
├── foo-sys/    ← 只做绑定（unsafe）
├── foo-safe/   ← 安全封装（推荐暴露给业务）
└── app/main.rs ← 只用 foo-safe
```

---

# 四、完整示例

---

## foo-sys（底层）

```rust
extern "C" {
    pub fn foo_add(a: i32, b: i32) -> i32;
}
```

---

## foo-safe（安全层）

```rust
pub fn add(a: i32, b: i32) -> i32 {
    unsafe { foo_sys::foo_add(a, b) }
}
```

---

## main.rs（业务）

```rust
fn main() {
    println!("{}", foo_safe::add(1, 2));
}
```

👉 **main 完全没有 unsafe**

---

# 五、进阶：更复杂封装（生产必备）

---

## 1️⃣ 错误处理

C：

```c
int foo_do(); // 返回 0=成功
```

Rust：

```rust
pub fn do_something() -> Result<(), String> {
    let ret = unsafe { foo_sys::foo_do() };
    if ret == 0 {
        Ok(())
    } else {
        Err("C error".into())
    }
}
```

---

## 2️⃣ 结构体封装

C：

```c
typedef struct foo_ctx foo_ctx;
foo_ctx* foo_new();
void foo_free(foo_ctx*);
```

Rust：

```rust
pub struct FooCtx {
    ptr: *mut foo_sys::foo_ctx,
}

impl Drop for FooCtx {
    fn drop(&mut self) {
        unsafe { foo_sys::foo_free(self.ptr) }
    }
}
```

👉 自动释放（RAII）

---

## 3️⃣ 线程安全控制

```rust
unsafe impl Send for FooCtx {}
unsafe impl Sync for FooCtx {}
```

👉 只有确认 C 是线程安全才加！

---

# 六、为什么这是 2025 主流架构

几乎所有 Rust FFI 项目都这么设计：

| 项目      | 分层                        |
| ------- | ------------------------- |
| openssl | openssl-sys + openssl     |
| sqlite  | libsqlite3-sys + rusqlite |
| libgit2 | libgit2-sys + git2        |

👉 规律：

```text
-sys = 绑定层
非-sys = 安全封装层
```

---

# 七、什么时候可以不写 foo-safe？

只有一种情况：

```text
你只是做实验 / demo / 一次性工具
```

否则：

> ❗生产代码必须有 foo-safe

---

# 八、你当前问题的本质

你之前遇到：

```rust
println!("{}", foo_safe::add(1, 2));
```

报错，本质是：

👉 你还没创建 `foo-safe` crate 或模块

---

# 九、最简实现 foo-safe（你现在就能用）

如果你不想拆 crate，可以先这样：

```rust
// src/foo_safe.rs

pub fn add(a: i32, b: i32) -> i32 {
    unsafe { foo_sys::foo_add(a, b) }
}
```

main.rs：

```rust
mod foo_safe;

fn main() {
    println!("{}", foo_safe::add(1, 2));
}
```

---

# 十、总结（非常重要）

```text
foo-sys  = 不安全（直接映射 C）
foo-safe = 安全（Rust 风格 API）
```

---

# 一句话结论

> 👉 **foo-safe 是 Rust FFI 在生产环境中“防止崩溃和内存问题”的安全边界**

---

如果你下一步想做“真正工程级”，我可以帮你升级：

* 🔥 workspace（foo-sys + foo-safe + app）
* 🔥 自动生成文档 + API 设计规范
* 🔥 FFI 零拷贝优化（性能级）
* 🔥 unsafe 最小化审计（类似大厂代码标准）

直接把你这个 demo 升级成 **可上线级 SDK 架构**。

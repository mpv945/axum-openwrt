use thiserror::Error;

#[derive(Error, Debug)]
pub enum FooError {
    #[error("null pointer")]
    NullPtr,
}

// c++ 添加方法 -> 安全调用的添加方法
pub fn sqrt(x: f64) -> f64 {
    foo_sys::sqrt(x)
}

pub fn add(a: i32, b: i32) -> i32 {
    foo_sys::add(a, b)
}

pub fn hello() -> Result<String, FooError> {
    Ok(foo_sys::hello())
}

pub fn alloc_string() -> Result<String, FooError> {
    let s = foo_sys::alloc_string();
    if s.is_empty() {
        return Err(FooError::NullPtr);
    }
    Ok(s)
}
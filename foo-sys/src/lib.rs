use std::ffi::{CStr};
use std::os::raw::c_char;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


// c++ 添加方法 -> 编译链接 foo_sqrt
pub fn sqrt(x: f64) -> f64 {
    unsafe { foo_sqrt(x) }
}

pub fn add(a: i32, b: i32) -> i32 {
    unsafe { foo_add(a, b) }
}

pub fn hello() -> String {
    unsafe {
        let ptr = foo_hello();
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

pub fn alloc_string() -> String {
    unsafe {
        let ptr = foo_alloc_string();
        if ptr.is_null() {
            return String::new();
        }

        let s = CStr::from_ptr(ptr).to_string_lossy().into_owned();
        foo_free(ptr as *mut c_char);
        s
    }
}
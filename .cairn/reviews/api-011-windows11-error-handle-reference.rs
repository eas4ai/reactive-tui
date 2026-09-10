use std::ffi::c_void;
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut c_void;
    fn GetProcessHandleCount(process: *mut c_void, count: *mut u32) -> i32;
}
fn count() -> u32 {
    let mut value = 0;
    assert_ne!(unsafe { GetProcessHandleCount(GetCurrentProcess(), &mut value) }, 0);
    value
}
fn main() {
    println!("Independent standard-library error-formatting probe");
    let before = count();
    println!("before: {before}");
    for attempt in 0..10 {
        let error = std::io::Error::from_raw_os_error(2).to_string();
        println!("attempt {attempt}: handles {}, error {error}", count());
    }
}

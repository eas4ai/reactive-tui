use std::{ffi::c_void, os::windows::{ffi::OsStrExt, io::{OwnedHandle, FromRawHandle, AsRawHandle}}};
#[repr(C)] #[derive(Clone, Copy)] struct Coord { x: i16, y: i16 }
type Create = unsafe extern "system" fn(Coord, *mut c_void, *mut c_void, u32, *mut isize) -> i32;
type Close = unsafe extern "system" fn(isize);
#[link(name = "kernel32")]
unsafe extern "system" {
 fn GetCurrentProcess() -> *mut c_void;
 fn GetProcessHandleCount(process: *mut c_void, count: *mut u32) -> i32;
 fn CreatePipe(read: *mut *mut c_void, write: *mut *mut c_void, attrs: *const c_void, size: u32) -> i32;
 fn CreatePseudoConsole(size: Coord, input: *mut c_void, output: *mut c_void, flags: u32, console: *mut isize) -> i32;
 fn ClosePseudoConsole(console: isize);
 fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
 fn GetProcAddress(module: *mut c_void, name: *const u8) -> Option<unsafe extern "system" fn() -> isize>;
 fn FreeLibrary(module: *mut c_void) -> i32;
}
fn count() -> u32 { let mut v=0; assert_ne!(unsafe {GetProcessHandleCount(GetCurrentProcess(), &mut v)},0); v }
fn pipe() -> (OwnedHandle,OwnedHandle) {
 let (mut r,mut w)=(std::ptr::null_mut(),std::ptr::null_mut());
 assert_ne!(unsafe{CreatePipe(&mut r,&mut w,std::ptr::null(),0)},0);
 unsafe{(OwnedHandle::from_raw_handle(r),OwnedHandle::from_raw_handle(w))}
}
fn cycles(label: &str, create: Create, close: Close) {
 println!("{label} before: {}",count());
 for n in 0..6 {
  let (r,w)=pipe(); let (o,p)=pipe(); let mut c=0;
  assert_eq!(unsafe{create(Coord{x:40,y:8},r.as_raw_handle(),p.as_raw_handle(),0,&mut c)},0);
  drop((r,w,o,p)); unsafe{close(c)};
  println!("{label} cycle {n}: {}",count());
 }
 std::thread::sleep(std::time::Duration::from_millis(500));
 println!("{label} after 500ms: {}",count());
}
fn main() {
 println!("Independent raw ConPTY probe; no Reactive-TUI code");
 println!("initial: {}",count());
 for n in 0..3 {
  let error=std::process::Command::new(std::env::current_dir().unwrap().join("missing-native-fixture.exe")).spawn().unwrap_err();
  println!("standard missing launch {n}: {} ({error})",count());
 }
 cycles("OS",CreatePseudoConsole,ClosePseudoConsole);
 let path=std::fs::canonicalize(std::env::args_os().nth(1).unwrap()).unwrap(); let wide:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();
 for load in 0..2 {
  let library=unsafe{LoadLibraryExW(wide.as_ptr(),std::ptr::null_mut(),0x1100)};
  assert!(!library.is_null(),"{}",std::io::Error::last_os_error());
  let create:Create=unsafe{std::mem::transmute(GetProcAddress(library,b"ConptyCreatePseudoConsole\0".as_ptr()).unwrap())};
  let close:Close=unsafe{std::mem::transmute(GetProcAddress(library,b"ConptyClosePseudoConsole\0".as_ptr()).unwrap())};
  cycles(&format!("SDK load {load}"),create,close);
  assert_ne!(unsafe{FreeLibrary(library)},0); println!("SDK unload {load}: {}",count());
 }
}

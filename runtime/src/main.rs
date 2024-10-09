/* use std::alloc::Layout;
use std::alloc::{alloc, dealloc};
use std::convert::TryInto;
use std::mem::size_of; */
#![no_main]
#![no_std]

use core::ffi::{c_char, CStr};

use libc_print::std_name::{eprintln, print, println};

// code for tick counter from
// tick_counter = "0.4.5"

// HACK: Some tool in our toolchain is dropping get_ticks_start
// on aarch64 because it is identical to get_ticks_end.
// So on aarch we call it something different
#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "aarch64")]
pub extern "C" fn _bril_get_ticks() -> u64 {
    use core::arch::asm;

    let tick_counter: u64;
    unsafe {
        asm!(
            "mrs x0, cntvct_el0",
            out("x0") tick_counter
        );
    }
    tick_counter
}

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub extern "C" fn _bril_get_ticks_start() -> u64 {
    let rax: u64;
    unsafe {
        asm!(
            "mfence",
            "lfence",
            "rdtsc",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") rax
        );
    }
    rax
}

#[no_mangle]
#[inline(never)]
#[cfg(target_arch = "x86_64")]
pub extern "C" fn _bril_get_ticks_end() -> u64 {
    let rax: u64;
    unsafe {
        asm!(
            "rdtsc",
            "lfence",
            "shl rdx, 32",
            "or rax, rdx",
            out("rax") rax
        );
    }
    rax
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_eprintln_unsigned_int(i: u64) {
    eprintln!("{}", i);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_print_int(i: i64) {
    print!("{}", i);
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_print_bool(b: bool) {
    if b {
        print!("true")
    } else {
        print!("false")
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_print_float(f: f64) {
    if f.is_infinite() {
        if f.is_sign_negative() {
            print!("-Infinity");
        } else {
            print!("Infinity");
        }
    } else if f.is_nan() {
        print!("NaN");
    } else if f == 0.0 {
        print!("{:.17}", 0.0)
    } else {
        print!("{:.17}", f);
    }
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_print_sep() {
    print!(" ");
}

#[no_mangle]
#[inline(never)]
pub extern "C" fn _bril_print_end() {
    println!();
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[inline(never)]
pub unsafe extern "C" fn _bril_parse_int(arg: *const c_char) -> i64 {
    let c_str = unsafe { CStr::from_ptr(arg) };
    let r_str = c_str.to_str().unwrap();
    r_str.parse::<i64>().unwrap()
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[inline(never)]
pub unsafe extern "C" fn _bril_parse_bool(arg: *const c_char) -> bool {
    let c_str = unsafe { CStr::from_ptr(arg) };
    let r_str = c_str.to_str().unwrap();
    r_str.parse::<bool>().unwrap()
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
#[inline(never)]
pub unsafe extern "C" fn _bril_parse_float(arg: *const c_char) -> f64 {
    let c_str = unsafe { CStr::from_ptr(arg) };
    let r_str = c_str.to_str().unwrap();
    r_str.parse::<f64>().unwrap()
}

#[cfg(not(test))]
#[inline(never)]
#[panic_handler]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

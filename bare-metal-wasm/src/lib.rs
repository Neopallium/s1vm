#![no_std]

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[inline(never)]
#[no_mangle]
pub extern fn inner_func(rdi: usize, rsi: usize, rdx: usize, rcx: usize, r8: usize) -> usize {
    rdi + rsi + rdx + rcx + r8
}

#[inline(never)]
#[no_mangle]
pub extern fn do_for_loop(rdi: usize, rsi: usize, rdx: usize, rcx: usize, r8: usize) -> usize {
    for idx in 0..rdi {
        if inner_func(idx, rsi, rdx, rcx, r8) == 0 {
            return 0;
        }
    }
    rdi
}

#[no_mangle]
pub extern fn for_loop(count: usize) -> usize {
  do_for_loop(count, count+1, count+2, count+3, count+4)
}

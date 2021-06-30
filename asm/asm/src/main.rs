#![feature(asm)]

use rustc_demangle::demangle;

extern "C" fn foo<T>() {
    println!("{}", std::any::type_name::<T>());
}

fn call_foo<T>() {
    unsafe {
        asm!(
            "call {}_ho",
            sym foo::<T>,
            // All caller-saved registers must be marked as clobbered
            out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _,
            out("r8") _, out("r9") _, out("r10") _, out("r11") _,
            out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
            out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
            out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
            out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
            // Also mark AVX-512 registers as clobbered. This is accepted by the
            // compiler even if AVX-512 is not enabled on the current target.
            out("xmm16") _, out("xmm17") _, out("xmm18") _, out("xmm19") _,
            out("xmm20") _, out("xmm21") _, out("xmm22") _, out("xmm23") _,
            out("xmm24") _, out("xmm25") _, out("xmm26") _, out("xmm27") _,
            out("xmm28") _, out("xmm29") _, out("xmm30") _, out("xmm31") _,
        )
    }
}

#[no_mangle]
extern "C" fn _RINvCs4VxPxZerBBV_7asm_asm3foolEB2__ho() {
    println!("{}", demangle("_RINvCs4VxPxZerBBV_7asm_asm3foolEB2_"))
}

#[no_mangle]
extern "C" fn _RINvCs4VxPxZerBBV_7asm_asm3fooNtNtCscRL1XZXviUR_5alloc6string6StringEB2__ho() {
    println!("{}", demangle("_RINvCs4VxPxZerBBV_7asm_asm3fooNtNtCscRL1XZXviUR_5alloc6string6StringEB2_"))
}

#[no_mangle]
extern "C" fn _RINvCs4VxPxZerBBV_7asm_asm3foomEB2__ho() {
    println!("{}", demangle("_RINvCs4VxPxZerBBV_7asm_asm3foomEB2_"))
}

fn main() {
    call_foo::<i32>();
    call_foo::<String>();
    call_foo::<u32>();
}

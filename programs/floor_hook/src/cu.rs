//!
//!
//!
//!
//!

///
#[cfg(feature = "cu-profile")]
#[inline(never)]
pub fn mark(label: &str) {
    anchor_lang::solana_program::log::sol_log(label);
    #[cfg(target_os = "solana")]
    unsafe {
        solana_define_syscall::definitions::sol_log_compute_units_()
    };
}

///
#[cfg(not(feature = "cu-profile"))]
#[inline(always)]
pub fn mark(_label: &str) {}

//! The TSC, the CPU's time-stamp counter, and its frequency as CPUID leaf
//! 0x15 gives it.

/// The TSC's count.
pub fn read() -> u64 {
    // SAFETY: Every x86_64 CPU has the TSC, and CR4.TSD restricts reading it
    // only outside ring 0. Reading it changes nothing.
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// The TSC's frequency: the crystal frequency in ECX times the ratio EBX /
/// EAX, or `None` where the leaf doesn't give all three.
pub fn hz() -> Option<u64> {
    if core::arch::x86_64::__cpuid(0).eax < 0x15 {
        return None;
    }
    let leaf = core::arch::x86_64::__cpuid(0x15);
    if leaf.eax == 0 || leaf.ebx == 0 || leaf.ecx == 0 {
        return None;
    }
    Some(u64::from(leaf.ecx) * u64::from(leaf.ebx) / u64::from(leaf.eax))
}

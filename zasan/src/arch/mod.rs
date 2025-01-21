#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "arm")]
mod arm;

#[cfg(target_arch = "powerpc")]
mod powerpc;

#[cfg(target_arch = "x86")]
mod x86;

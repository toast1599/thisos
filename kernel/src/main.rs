//! The ThisOS kernel binary: the boot loader's entry into the library.

#![no_std]
#![no_main]
#![forbid(unsafe_code)]

thisos_kernel::entry!(thisos_kernel::kmain, thisos_kernel::trap);

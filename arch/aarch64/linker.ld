/* Links the aarch64 kernel at the top 2 GiB, where Limine expects a kernel
   that isn't relocatable, with one segment per permission so that Limine
   maps code read-execute, read-only data read-only and the rest
   read-write. */

OUTPUT_FORMAT(elf64-littleaarch64)
ENTRY(_start)

PHDRS
{
    text   PT_LOAD FLAGS(5);
    rodata PT_LOAD FLAGS(4);
    data   PT_LOAD FLAGS(6);
    stack  PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        __text_start = .;
        *(.text._start)
        *(.text .text.*)
        __text_end = .;
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .data : {
        *(.data .data.*)
        *(.data.rel.ro .data.rel.ro.*)
        *(.got .got.*)
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /* The boot stack, in a segment of its own at the alignment its section
       asks for, at least one page past the end of the data. Limine maps
       nothing in between, so that page is the stack's guard. */
    . = ALIGN(4096) + 4096;
    .boot_stack (NOLOAD) : {
        *(.boot_stack)
    } :stack

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}

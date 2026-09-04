ENTRY(_start)

PHDRS {
  image PT_LOAD FILEHDR PHDRS FLAGS(7);
}

SECTIONS {
  . = 0x400000 + SIZEOF_HEADERS;

  .text : {
    *(.text .text.*)
    *(.rodata .rodata.*)
  } :image

  .data : {
    *(.data .data.*)
  } :image

  .bss (NOLOAD) : {
    *(.bss .bss.*)
    *(COMMON)
  } :image

  /DISCARD/ : {
    *(.comment*)
    *(.eh_frame*)
    *(.note*)
    *(.gnu*)
  }
}


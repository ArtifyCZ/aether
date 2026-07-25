use core::arch::asm;

pub unsafe fn inb(port: u16) -> u8 {
    unsafe {
        let ret: u8;
        asm!(
            "inb %dx, %al",
            out("al") ret,
            in("dx") port,
            options(att_syntax),
        );
        ret
    }
}

pub unsafe fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "outb %al, %dx",
            in("al") value,
            in("dx") port,
            options(att_syntax),
        );
    }
}

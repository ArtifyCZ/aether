#![no_std]

use log::info;

pub fn bsp_main() -> ! {
    info!("Booting Atom kernel...");
    info!("Bootstrap core starting...");
    let chars = ['a', 'b', 'c', 'd', 'e', 'f'];
    for i in 0..150 {
        let c = chars[i % chars.len()];
        info!("{} - {}", i, c);
        for _ in 0..1000000 {
            core::hint::spin_loop();
        }
    }
    loop {}
}

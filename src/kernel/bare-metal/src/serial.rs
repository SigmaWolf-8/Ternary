// Copyright (c) 2025-2026 Capomastro Holdings Ltd. (Canada)
// Applied Physics Division — Salvi Framework
// PROPRIETARY AND CONFIDENTIAL — All Rights Reserved

//! Bare-metal serial output driver — COM1 (0x3F8)
//! QEMU captures this with -serial stdio

use spin::Mutex;

const COM1_PORT: u16 = 0x3F8;
static SERIAL_INIT: Mutex<bool> = Mutex::new(false);

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, nomem));
}

#[inline(always)]
unsafe fn inb(port: u16) -> u8 {
    let val: u8;
    core::arch::asm!("in al, dx", in("dx") port, out("al") val, options(nostack, nomem));
    val
}

pub fn init() {
    let mut initialized = SERIAL_INIT.lock();
    if *initialized { return; }
    unsafe {
        outb(COM1_PORT + 1, 0x00);
        outb(COM1_PORT + 3, 0x80);
        outb(COM1_PORT + 0, 0x01);
        outb(COM1_PORT + 1, 0x00);
        outb(COM1_PORT + 3, 0x03);
        outb(COM1_PORT + 2, 0xC7);
        outb(COM1_PORT + 4, 0x0B);
        outb(COM1_PORT + 4, 0x1E);
        outb(COM1_PORT + 0, 0xAE);
        let _ = inb(COM1_PORT + 0);
        outb(COM1_PORT + 4, 0x0F);
    }
    *initialized = true;
}

fn wait_transmit_ready() {
    while unsafe { inb(COM1_PORT + 5) } & 0x20 == 0 {
        core::hint::spin_loop();
    }
}

pub fn write_byte(byte: u8) {
    wait_transmit_ready();
    unsafe { outb(COM1_PORT, byte); }
}

pub fn print_str(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' { write_byte(b'\r'); }
        write_byte(byte);
    }
}

pub fn print_line(s: &str) {
    print_str(s);
    write_byte(b'\r');
    write_byte(b'\n');
}

pub fn print_u64(mut val: u64) {
    if val == 0 { write_byte(b'0'); return; }
    let mut buf = [0u8; 20];
    let mut pos = 0;
    while val > 0 {
        buf[pos] = b'0' + (val % 10) as u8;
        val /= 10;
        pos += 1;
    }
    for i in (0..pos).rev() { write_byte(buf[i]); }
}

pub fn print_i32(val: i32) {
    if val < 0 {
        write_byte(b'-');
        print_u64((-val) as u64);
    } else {
        print_u64(val as u64);
    }
}

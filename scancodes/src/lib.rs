#![no_std]

static PS2_SET1: [u16; 89] = [
    0,
    0x1B,
    '1' as u16,
    '2' as u16,
    '3' as u16,
    '4' as u16,
    '5' as u16,
    '6' as u16,
    '7' as u16,
    '8' as u16,
    '9' as u16,
    '0' as u16,
    '-' as u16,
    '=' as u16,
    '\x08' as u16,
    '\t' as u16,
    'Q' as u16,
    'W' as u16,
    'E' as u16,
    'R' as u16,
    'T' as u16,
    'Y' as u16,
    'U' as u16,
    'I' as u16,
    'O' as u16,
    'P' as u16,
    '[' as u16,
    ']' as u16,
    '\n' as u16,
    0x0100, // KeyCtrlLeft
    'A' as u16,
    'S' as u16,
    'D' as u16,
    'F' as u16,
    'G' as u16,
    'H' as u16,
    'J' as u16,
    'K' as u16,
    'L' as u16,
    ';' as u16,
    '\'' as u16,
    '`' as u16,
    0x0200, // KeyShiftLeft
    '\\' as u16,
    'Z' as u16,
    'X' as u16,
    'C' as u16,
    'V' as u16,
    'B' as u16,
    'N' as u16,
    'M' as u16,
    ',' as u16,
    '.' as u16,
    '/' as u16,
    0x0300, // KeyShiftRight
    0x0400, // KeyPad('*')
    0x0500, // KeyAltLeft
    ' ' as u16,
    0x0600, // KeyCapsLock
    0x0701, // KeyFn(1)
    0x0702, // KeyFn(2)
    0x0703, // KeyFn(3)
    0x0704, // KeyFn(4)
    0x0705, // KeyFn(5)
    0x0706, // KeyFn(6)
    0x0707, // KeyFn(7)
    0x0708, // KeyFn(8)
    0x0709, // KeyFn(9)
    0x070A, // KeyFn(10)
    0x0800, // KeyNumLock
    0x0900, // KeyScrollLock
    0x0417, // KeyPad('7')
    0x0418, // KeyPad('8')
    0x0419, // KeyPad('9')
    0x041A, // KeyPad('-')
    0x0414, // KeyPad('4')
    0x0415, // KeyPad('5')
    0x0416, // KeyPad('6')
    0x041B, // KeyPad('+')
    0x0411, // KeyPad('1')
    0x0412, // KeyPad('2')
    0x0413, // KeyPad('3')
    0x0410, // KeyPad('0')
    0x041C, // KeyPad('.')
    0,
    0,
    0,
    0x070B, // KeyFn(11)
    0x070C, // KeyFn(12)
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keysym(u16);
impl Keysym {
    pub fn from(key: u16) -> Self {
        Keysym(key)
    }

    pub fn key_unknown() -> Self {
        Keysym(0)
    }

    pub fn is_unknown(&self) -> bool {
        self.0 == 0
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

pub fn scancode_to_keysym(scancode: u8) -> Keysym {
    if scancode as usize >= PS2_SET1.len() {
        return Keysym::key_unknown();
    }
    let key = PS2_SET1[scancode as usize];
    if key == 0 {
        return Keysym::key_unknown();
    }
    Keysym::from(key)
}

pub fn scancode_to_ascii(scancode: u8) -> Option<u8> {
    if scancode as usize >= PS2_SET1.len() {
        return None;
    }
    let key = PS2_SET1[scancode as usize];
    if (0x20..=0x7E).contains(&key) || matches!(key, 0x08 | 0x09 | 0x0A | 0x0D) {
        Some(key as u8)
    } else {
        None
    }
}

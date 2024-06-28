use crate::stack;

pub struct Chip8 {
    pub scale: i32,
    pub mem: [u8; 4096],
    pub width: i32,
    pub height: i32,
    pub pc: u16,
    pub I: u16,
    pub stack: stack::Stack,
    pub timer_delay: u8,
    pub timer_sound: u8,

    // regs
    pub V0: u8,
    pub V1: u8,
    pub V2: u8,
    pub V3: u8,
    pub V4: u8,
    pub V5: u8,
    pub V6: u8,
    pub V7: u8,
    pub V8: u8,
    pub V9: u8,
    pub VA: u8,
    pub VB: u8,
    pub VC: u8,
    pub VD: u8,
    pub VE: u8,
    pub VF: u8,  // flag reg
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            scale: 10,
            mem: [0; 4096],
            width: 64,
            height: 32,
            pc: 0,
            I: 0,
            stack: stack::Stack::new(),
            timer_delay: 255,
            timer_sound: 255,
            V0: 0,
            V1: 0,
            V2: 0,
            V3: 0,
            V4: 0,
            V5: 0,
            V6: 0,
            V7: 0,
            V8: 0,
            V9: 0,
            VA: 0,
            VB: 0,
            VC: 0,
            VD: 0,
            VE: 0,
            VF: 0,
        }
    }
}
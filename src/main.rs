extern crate beryllium;
extern crate gl33;

use beryllium::events::{Event, SDL_Scancode};
use beryllium::*;
use beryllium::init::*;
use beryllium::video::{CreateWinArgs, GlWindow};

use gl33::*;
use gl33::global_loader::{glBindBuffer, glClear, glClearColor, glGenBuffers, glGenVertexArrays};
use chip8::Chip8;

mod lib;
mod stack;
mod chip8;

// return error when an error accurs lmao
fn execute_instruction(chip8: &mut Chip8, window: &mut GlWindow) {
    // fetch
    chip8.mem[chip8.pc as usize] = 0b00111111;
    chip8.mem[chip8.pc as usize +1] = 0b00111100;
    let instruct: u16 = ((chip8.mem[chip8.pc as usize] as u16) << 8) + chip8.mem[(chip8.pc as usize)+1] as u16;
    chip8.pc += 2;
    // decode & execute
    let nib_0 = (instruct >> 12) & 0xF;
    let nib_1 = (instruct >> 8) & 0xF;
    let nib_2 = (instruct >> 4) & 0xF;
    let nib_3 = instruct & 0xF;
    match nib_0 {
        0x0 => {
            match nib_1 {
                0x0 => {
                    match nib_2 {
                        0xE => {
                            match nib_3 {
                                0x0 => {
                                    // clear screen
                                    unsafe {
                                        glClear(GL_COLOR_BUFFER_BIT);
                                    }
                                }
                                0xE => {
                                    // return from subroutine
                                    chip8.pc = chip8.stack.pop();
                                }
                                _ => println!("command not implemented"),
                            }
                        }
                        _ => println!("command not implemented"),
                    }
                }
                _ => println!("command not implemented"),
            }
        },
        0x1 => {
            // jmp
            chip8.pc = (nib_1 << 8) + (nib_2 << 4) + nib_3;
        },
        0x2 => {
            // go to subroutine
            chip8.stack.push(chip8.pc);

        },
        0x3 => {
            // skips if VX == NN
            let v0_ptr = std::ptr::addr_of!(chip8.V0);
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                if *vx_ptr == ((nib_2 << 4) + nib_3) as u8 {
                    chip8.pc += 2;
                };
            }
        },
        0x4 => {
            // skips if VX != NN
            let v0_ptr = std::ptr::addr_of!(chip8.V0);
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                if *vx_ptr != ((nib_2 << 4) + nib_3) as u8 {
                    chip8.pc += 2;
                };
            }
        },
        0x5 => {
            // skips if VX == VY
            let v0_ptr = std::ptr::addr_of!(chip8.V0);
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                if *vx_ptr == *vy_ptr {
                    chip8.pc += 2;
                };
            }
        },
        0x6 => {
            // set VX to NN
            let v0_ptr: *mut u8 = &mut chip8.V0;
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                *vx_ptr = ((nib_2 << 4) + nib_3) as u8;
            }

        },
        0x7 => {
            // add NN to VX
            let v0_ptr: *mut u8 = &mut chip8.V0;
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                *vx_ptr += ((nib_2 << 4) + nib_3) as u8;
            }
        },
        0x8 => {
            match nib_3 {
                0x0 => {
                    // set VX to VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr == *vy_ptr;
                    }
                },
                0x1 => {
                    // set VX to VX | VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr |= *vy_ptr;
                    }
                },
                0x2 => {
                    // set VX to VX | VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr &= *vy_ptr;
                    }
                },
                0x3 => {
                    // set VX to VX | VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr ^= *vy_ptr;
                    }
                },
                0x4 => {
                    // set VX to VX + VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        match *vx_ptr.checked_add(*vy_ptr) {
                            Some(_) => {
                                *vx_ptr += *vy_ptr;
                                chip8.VF = 0;
                            },
                            None => {
                                *vx_ptr = *vx_ptr.wrapping_add(*vy_ptr as usize);
                                chip8.VF = 1;
                            }
                        };
                    }
                },
                0x5 => {
                    // set VX to VX - VY
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        match *vx_ptr.checked_sub(*vy_ptr) {
                            Some(_) => {
                                *vx_ptr -= *vy_ptr;
                                chip8.VF = 1;
                            },
                            None => {
                                *vx_ptr = *vx_ptr.wrapping_sub(*vy_ptr as usize);
                                chip8.VF = 0;
                            }
                        };
                    }
                },
                0x6 => {
                    // TODO: make configurable by user which version to use
                    // set VX to VY, shift by 1 to right
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr = *vy_ptr;
                        chip8.VF = *vx_ptr & 1;
                        *vx_ptr >>= 1;
                    }
                },
                0x7 => {
                    // set VX to VY - VX
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        match *vy_ptr.checked_sub(*vx_ptr) {
                            Some(_) => {
                                *vy_ptr -= *vx_ptr;
                                chip8.VF = 1;
                            },
                            None => {
                                *vx_ptr = *vy_ptr.wrapping_sub(*vx_ptr as usize);
                                chip8.VF = 0;
                            }
                        };
                    }
                },
                0xE => {
                    // TODO: make configurable by user which version to use
                    // set VX to VY, shift by 1 to right
                    let v0_ptr: *mut u8 = &mut chip8.V0;
                    unsafe {
                        let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                        let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                        *vx_ptr = *vy_ptr;
                        chip8.VF = (*vx_ptr >> 7) & 1;
                        *vx_ptr <<= 1;
                    }
                },
                _ => println!("command not implemented"),
            }
        }
        0x9 => {
            // skips if VX != VY
            let v0_ptr = std::ptr::addr_of!(chip8.V0);
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                if *vx_ptr != *vy_ptr {
                    chip8.pc += 2;
                };
            }
        },
        0xA => {
            // set I to NNN
            chip8.I = (nib_1 << 8) + (nib_2 << 4) + nib_3;
        },
        0xB => {
            // TODO: make configurable by user which version to use
            // pc = NNN + V0
            chip8.pc = chip8.V0 as u16 + ((nib_1 << 8) + (nib_2 << 4) + nib_3);
        },
        0xC => {
            // set VX to rand num & NN
            let v0_ptr: *mut u8 = &mut chip8.V0;
            unsafe {
                let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            }
        },
        _ => println!("command not implemented"),
    }
}

fn main() {
    println!("Hello, world!");

    let mut chip8 = Chip8::new();

    // Font
    let font: [u8; 80] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];
    chip8.mem[0x50..0xA0].clone_from_slice(&font);
    println!("{}", chip8.mem[0x50]);
    // Window
    let sdl = Sdl::init(InitFlags::EVERYTHING);

    let win_args = CreateWinArgs{
        title: "Chip-8",
        width: chip8.width*chip8.scale,
        height: chip8.height*chip8.scale,
        allow_high_dpi: false,
        borderless: false,
        resizable: false,
    };
    let mut window = sdl.create_gl_window(
        win_args,
    ).expect("Failed to make window :(");

    // unsafe {
    //     let mut vao = 0;
    //     glGenVertexArrays(1, &mut vao);
    //
    //     let mut vb0 = 0;
    //     glGenBuffers(1, &mut vb0);
    //
    //     glBindBuffer(GL_ARRAY_BUFFER, vb0);
    // }

    'main_loop: loop {
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                Event::Quit => break 'main_loop,
                Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                    println!("{:?}, {:?}, {:?}", scancode, keycode, win_id);
                    match scancode.0 {
                        20 => println!("q"),
                        26 => println!("w"),
                        8  => println!("e"),
                        21 => println!("r"),
                        _ => println!("other scancode"),
                    }
                    execute_instruction(&mut chip8, &mut window)  // TODO: fix timing, run at 60fps
                }
                _ => (),
            }
        }
    }
}
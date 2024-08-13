extern crate beryllium;

use std::ffi::c_int;
use beryllium::events::{Event, SDL_Scancode};
use beryllium::*;
use beryllium::init::*;
use beryllium::video::{CreateWinArgs, GlWindow, RendererFlags, RendererWindow, RendererInfo};

use rand::Rng;
use chip8::Chip8;


mod lib;
mod stack;
mod chip8;

fn sdl_draw(chip8: &Chip8, renderer: &RendererWindow) {
    renderer.set_draw_color(0, 0, 0, 255).unwrap();
    renderer.clear().unwrap();
    renderer.set_draw_color(255, 255, 255, 255).unwrap();
    let scale: c_int = chip8.scale;
    for (i, row) in chip8.bitmap.iter().enumerate() {
        for (j, &pixel) in row.iter().enumerate() {
            if pixel == 0 {continue}
            let x: c_int = j as c_int;
            let y: c_int = i as c_int;
            let mut p1: [c_int; 2];
            let mut p2: [c_int; 2];
            for k in 0..chip8.scale {
                p1 = [x*scale, y*scale+k];
                p2 = [x*scale+scale, y*scale+k];
                let points: [[c_int; 2]; 2] = [p1, p2];
                renderer.draw_lines(&points).expect("nope");
                renderer.present();
            }
        }
    }
}


// return error when an error occurs lmao
fn execute_instruction(chip8: &mut Chip8, renderer: &mut RendererWindow, sdl: &Sdl) {
    // fetch
    chip8.mem[chip8.pc as usize] = 0xF2;
    chip8.mem[chip8.pc as usize +1] = 0x0A;
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
                                    chip8.bitmap = [[0; 64]; 32];
                                    sdl_draw(&chip8, &renderer);
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
            // let v0_ptr = std::ptr::addr_of!(chip8.V0);
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     if *vx_ptr == ((nib_2 << 4) + nib_3) as u8 {
            //         chip8.pc += 2;
            //     };
            // }
            if chip8.V[nib_1 as usize] == ((nib_2 << 4) + nib_3) as u8 {
                chip8.pc += 2;
            };
        },
        0x4 => {
            // skips if VX != NN
            // let v0_ptr = std::ptr::addr_of!(chip8.V0);
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     if *vx_ptr != ((nib_2 << 4) + nib_3) as u8 {
            //         chip8.pc += 2;
            //     };
            // }
            if chip8.V[nib_1 as usize] != ((nib_2 << 4) + nib_3) as u8 {
                chip8.pc += 2;
            };
        },
        0x5 => {
            // skips if VX == VY
            // let v0_ptr = std::ptr::addr_of!(chip8.V0);
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
            //     if *vx_ptr == *vy_ptr {
            //         chip8.pc += 2;
            //     };
            // }
            if chip8.V[nib_1 as usize] == chip8.V[nib_2 as usize] {
                chip8.pc += 2;
            }
        },
        0x6 => {
            // set VX to NN
            // let v0_ptr: *mut u8 = &mut chip8.V0;
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     *vx_ptr = ((nib_2 << 4) + nib_3) as u8;
            // }
            chip8.V[nib_1 as usize] = ((nib_2 << 4) + nib_3) as u8;
        },
        0x7 => {
            // add NN to VX
            // let v0_ptr: *mut u8 = &mut chip8.V0;
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     *vx_ptr += ((nib_2 << 4) + nib_3) as u8;
            // }
            chip8.V[nib_1 as usize] += ((nib_2 << 4) + nib_3) as u8;
        },
        0x8 => {
            match nib_3 {
                0x0 => {
                    // set VX to VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr = *vy_ptr;
                    // }
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize]
                },
                0x1 => {
                    // set VX to VX | VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr |= *vy_ptr;
                    // }
                    chip8.V[nib_1 as usize] |= chip8.V[nib_2 as usize];
                },
                0x2 => {
                    // set VX to VX | VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr &= *vy_ptr;
                    // }
                },
                0x3 => {
                    // set VX to VX | VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr ^= *vy_ptr;
                    // }
                    chip8.V[nib_1 as usize] ^= chip8.V[nib_2 as usize];
                },
                0x4 => {
                    // set VX to VX + VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     let vx_as_num = *vx_ptr;
                    //     let vy_as_num = *vy_ptr;
                    //     match vx_as_num.checked_add(vy_as_num) {
                    //         Some(_) => {
                    //             *vx_ptr += *vy_ptr;
                    //             chip8.VF = 0;
                    //         },
                    //         None => {
                    //             *vx_ptr = *vx_ptr.wrapping_add(*vy_ptr as usize);
                    //             chip8.VF = 1;
                    //         }
                    //     };
                    // }
                    match chip8.V[nib_1 as usize].checked_add(chip8.V[nib_2 as usize]) {
                        Some(_) => {
                            chip8.V[nib_1 as usize] += chip8.V[nib_2 as usize];
                            chip8.V[0xF] = 0;
                        },
                        None => {
                            chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize].wrapping_add(chip8.V[nib_2 as usize]);
                            chip8.V[0xF] = 1;
                        }
                    }
                },
                0x5 => {
                    // set VX to VX - VY
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     let vx_as_num = *vx_ptr;
                    //     let vy_as_num = *vy_ptr;
                    //     match vx_as_num.checked_sub(vy_as_num) {
                    //         Some(_) => {
                    //             *vx_ptr -= *vy_ptr;
                    //             chip8.VF = 1;
                    //         },
                    //         None => {
                    //             *vx_ptr = *vx_ptr.wrapping_sub(*vy_ptr as usize);
                    //             chip8.VF = 0;
                    //         }
                    //     };
                    // }
                    match chip8.V[nib_1 as usize].checked_sub(chip8.V[nib_2 as usize]) {
                        Some(_) => {
                            chip8.V[nib_1 as usize] -= chip8.V[nib_2 as usize];
                            chip8.V[0xF] = 0;
                        },
                        None => {
                            chip8.V[nib_1 as usize] = chip8.V[nib_1 as usize].wrapping_sub(chip8.V[nib_2 as usize]);
                            chip8.V[0xF] = 1;
                        }
                    }
                },
                0x6 => {
                    // TODO: make configurable by user which version to use
                    // set VX to VY, shift by 1 to right
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr = *vy_ptr;
                    //     chip8.VF = *vx_ptr & 1;
                    //     *vx_ptr >>= 1;
                    // }
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize] >> 1;
                },
                0x7 => {
                    // set VX to VY - VX
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     let vx_as_num = *vx_ptr;
                    //     let vy_as_num = *vy_ptr;
                    //     match vy_as_num.checked_sub(vx_as_num) {
                    //         Some(_) => {
                    //             *vx_ptr = *vy_ptr - *vx_ptr;
                    //             chip8.VF = 1;
                    //         },
                    //         None => {
                    //             *vx_ptr = *vy_ptr.wrapping_sub(*vx_ptr as usize);
                    //             chip8.VF = 0;
                    //         }
                    //     };
                    // }
                    match chip8.V[nib_2 as usize].checked_sub(chip8.V[nib_1 as usize]) {
                        Some(_) => {
                            chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize] - chip8.V[nib_1 as usize];
                            chip8.V[0xF] = 1;
                        },
                        None => {
                            chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize].wrapping_sub(chip8.V[nib_1 as usize]);
                            chip8.V[0xF] = 0;
                        }
                    }
                },
                0xE => {
                    // TODO: make configurable by user which version to use
                    // set VX to VY, shift by 1 to right
                    // let v0_ptr: *mut u8 = &mut chip8.V0;
                    // unsafe {
                    //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
                    //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
                    //     *vx_ptr = *vy_ptr;
                    //     chip8.VF = (*vx_ptr >> 7) & 1;
                    //     *vx_ptr <<= 1;
                    // }
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize];
                    chip8.V[0xF] = (chip8.V[nib_1 as usize] > 7) as u8 & 1;
                    chip8.V[nib_1 as usize] << 1;
                },
                _ => println!("command not implemented"),
            }
        }
        0x9 => {
            // skips if VX != VY
            // let v0_ptr = std::ptr::addr_of!(chip8.V0);
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     let vy_ptr = v0_ptr.offset((nib_2 as u8) as isize);
            //     if *vx_ptr != *vy_ptr {
            //         chip8.pc += 2;
            //     };
            // }
            if chip8.V[nib_1 as usize] != chip8.V[nib_2 as usize] {
                chip8.pc += 2;
            }
        },
        0xA => {
            // set I to NNN
            chip8.I = (nib_1 << 8) + (nib_2 << 4) + nib_3;
        },
        0xB => {
            // TODO: make configurable by user which version to use
            // pc = NNN + V0
            chip8.pc = chip8.V[0] as u16 + ((nib_1 << 8) + (nib_2 << 4) + nib_3);
        },
        0xC => {
            // set VX to rand num & NN
            // let v0_ptr: *mut u8 = &mut chip8.V0;
            let NN:u8 = ((nib_2 << 4) + nib_3) as u8;
            let mut rng = rand::thread_rng();
            let rand_num: u8 = (rng.gen::<u8>());
            // unsafe {
            //     let vx_ptr = v0_ptr.offset((nib_1 as u8) as isize);
            //     *vx_ptr = (rand_num & NN);
            // }
            chip8.V[nib_1 as usize] = (rand_num & NN);
        },
        0xD => {
            // TODO: this
            // display
            // draws N pixels tall sprite from mem location that I reg is holding to the screen
            // VX, VY coords
            // all pixels that are "on" will flip pixels on screen
            // drawn left to right, most to least significant bit
            // if any pixels turned "off", set VF flag to 1, else set to 0
            let x: u8;
            let y: u8;
            x = chip8.V[nib_1 as usize] & chip8.width as u8 - 1;
            y = chip8.V[nib_2 as usize] & chip8.height as u8 - 1;
            chip8.V[0xF] = 0;
            set_bitmap(chip8, x, y, nib_3, &renderer);
        },
        0xE => {
            // TODO: key stuff
            match nib_2 {
                0x9 => {
                    match nib_3 {
                        0xE => {
                            // Checks if key is held down, PC += 2

                        }
                        _ => println!("command not implemented"),
                    }
                }
                0xA => {
                    match nib_3 {
                        1 => {
                            // Checks if key is held down, PC += 2 if not same in vx or if not held down
                        },
                        _ => println!("command not implemented"),
                    }
                }
                _ => println!("command not implemented"),
            }
        },
        0xF => {
            match nib_2 {
                0 => {
                    match nib_3 {
                        7 => {
                            // Sets VX to current value of delay timer
                            chip8.V[nib_1 as usize] = chip8.timer_delay as u8
                        },
                        0xA => {
                            // Waits for key
                            println!("waiting for key");
                            loop {
                                match sdl.poll_events() {
                                    Some((event, _timestamp)) => {
                                        match event {
                                            Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                                                println!("{:?}, {:?}, {:?}", scancode, keycode, win_id);
                                                chip8.V[nib_1 as usize] = keycode.0 as u8;
                                                println!("Keycode: {:?}", keycode)
                                            }
                                            _ => {
                                                chip8.pc -= 2;
                                                break;
                                            },
                                        }
                                    }
                                    None => (),
                                }
                                println!("{} {}", chip8.pc, chip8.V[nib_1 as usize])
                            }

                        },
                        _ => println!("Command not implemented"),
                    }
                },
                1 => {
                    match nib_3 {
                        5 => {
                            // Set delay timer to value in VX
                            chip8.timer_delay = chip8.V[nib_1 as usize];
                        },
                        9 => {

                        },
                        8 => {
                            // Set sound timer to value in VX
                            chip8.timer_sound = chip8.V[nib_1 as usize];
                        },
                        0xE => {
                            // Add value in VX to I
                            match chip8.I.checked_add(chip8.V[nib_1 as usize] as u16) {
                                Some(_) => {
                                    chip8.I += chip8.V[nib_1 as usize] as u16;
                                },
                                None => {
                                    chip8.I = chip8.I.wrapping_add(chip8.V[nib_1 as usize] as u16);
                                    chip8.V[0xF] = 1;
                                }
                            }
                            chip8.I += chip8.V[nib_1 as usize] as u16;
                        },
                        _ => println!("Command not implemented"),
                    }
                },
                2 => {
                    match nib_3 {
                        9 => {

                        },
                        _ => println!("Command not implemented"),
                    }
                }
                _ => println!("Command not implemented"),
            }
        }
        _ => println!("command not implemented"),
    }
}

fn set_bitmap(chip8: &mut Chip8, mut x: u8, mut y: u8, n: u16, renderer: &RendererWindow) {
    renderer.set_draw_color(0, 0, 0, 255).unwrap();
    renderer.clear().unwrap();
    renderer.set_draw_color(255, 255, 255, 255).unwrap();
    let scale: c_int = chip8.scale;
    let mut current_row: i32 = 0;
    let mut current_byte: i32 = 0;
    let max_row_bytes = chip8.width/8;
        for i in 0..n {
            let byte = chip8.mem[i as usize];
            for j in 0..8 {
                let bit = (byte >> i) & 1;
                if chip8.bitmap[x as usize][y as usize] == 1 && bit == 1 {
                    chip8.V[0xF] = 1;
                }
                chip8.bitmap[x as usize][y as usize] | bit;
                x += 1;
            }
            current_byte += 1;
            if current_row == chip8.height && current_byte == max_row_bytes {
                return;
            }
            if current_byte == max_row_bytes {
                current_byte = 0;
                current_row += 1;
                y += 1;
            }
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
    //let mut window = sdl.create_gl_window(
    //    win_args,
    //).expect("Failed to make window :(");
    let win_args = CreateWinArgs{
        title: "Chip-8",
        width: chip8.width*chip8.scale,
        height: chip8.height*chip8.scale,
        allow_high_dpi: false,
        borderless: false,
        resizable: false,
    };
    let render_flags: RendererFlags = <RendererFlags as std::default::Default>::default();
    let mut renderer = sdl.create_renderer_window(win_args, render_flags,).unwrap();

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
        // let mut bitmap: [[u16; 64]; 32] = [[0; 64]; 32];
        // let mut rng = rand::thread_rng();
        // for i in 0..bitmap.len() {
        //     for j in 0..bitmap[i].len() {
        //         let random_number: u16 = rng.gen_range(0..=1);
        //         bitmap[i][j] = random_number;
        //     }
        // }
        //sdl_draw(&chip8, &bitmap, &renderer);
        while let Some((event, _timestamp)) = sdl.poll_events() {
            match event {
                Event::Quit => break 'main_loop,
                // Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                //     println!("{:?}, {:?}, {:?}", scancode, keycode, win_id);
                //     match scancode.0 {
                //         20 => println!("q"),
                //         26 => println!("w"),
                //         8 => println!("e"),
                //         21 => println!("r"),
                //         _ => println!("other scancode"),
                //     }

                //}
                _ => execute_instruction(&mut chip8, &mut renderer, &sdl)  // TODO: fix timing, run at 60fps,
            }
        }
    }
}
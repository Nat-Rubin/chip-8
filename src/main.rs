extern crate beryllium;

use std::ffi::c_int;
use std::io::BufRead;
use std::env::consts::OS;
use beryllium::events::{Event, SDL_Scancode};
use beryllium::*;
use beryllium::init::*;
use beryllium::video::{CreateWinArgs, GlWindow, RendererFlags, RendererWindow, RendererInfo};
use egui::Key::P;
use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use chip8::Chip8;

use errors::Error;
use beeps::*;

mod stack;
mod chip8;
mod errors;
mod beeps;

fn exit_with_error(chip8: &Chip8, error: Error, instruction: u16, ) {
    match error {
        Error::UnknownInstruction => {
            println!("Error: UnknownInstruction");
            println!("Instruction {} does not exist or is not yet implemented.", instruction)
        },
        Error::NoFileGiven => {
            println!("Error: NoFileGiven");
            println!("No file provided for the program.");
        },
        Error::FileNotFound => {
            println!("Error: FileNotFound");
            println!("The file path you provided does not exist or cannot be found.");
        }
        _ => (),
    }

    std::process::exit(0)
}

fn sdl_draw(chip8: &Chip8, renderer: &RendererWindow) {
    renderer.set_draw_color(0, 0, 0, 255).unwrap();
    renderer.clear().unwrap();

    renderer.set_draw_color(255, 255, 255, 255).unwrap();
    let scale: c_int = chip8.scale;
    for (i, row) in chip8.bitmap.iter().enumerate() {  // current row (y)
        for (j, &pixel) in row.iter().enumerate() {  // current pixel in row (x)
            if pixel == 0 {continue}
            let x: c_int = j as c_int;
            let y: c_int = i as c_int;
            let mut p1: [c_int; 2];
            let mut p2: [c_int; 2];
            for k in 0..chip8.scale {
                p1 = [x*scale, y*scale+k];
                p2 = [x*scale+scale, y*scale+k];
                let points: [[c_int; 2]; 2] = [p1, p2];
                renderer.draw_lines(&points).expect("renderer is not drawing lines");
                // renderer.present(); might have to be here if it builds too slowly
            }
        }
    }
    renderer.present();
}


// return error when an error occurs lmao
fn execute_instruction(chip8: &mut Chip8, renderer: &mut RendererWindow, sdl: &Sdl, instruct: u16) {
    // fetch
    // chip8.mem[chip8.pc as usize] = 0xF2;
    // chip8.mem[chip8.pc as usize +1] = 0x0A;
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
                                _ => exit_with_error(chip8, Error::UnknownInstruction, instruct)
                            }
                        }
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct)
                    }
                }
                _ => exit_with_error(chip8, Error::UnknownInstruction, instruct)
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
            if chip8.V[nib_1 as usize] == ((nib_2 << 4) + nib_3) as u8 {
                chip8.pc += 2;
            };
        },
        0x4 => {
            // skips if VX != NN
            if chip8.V[nib_1 as usize] != ((nib_2 << 4) + nib_3) as u8 {
                chip8.pc += 2;
            };
        },
        0x5 => {
            // skips if VX == VY
            if chip8.V[nib_1 as usize] == chip8.V[nib_2 as usize] {
                chip8.pc += 2;
            }
        },
        0x6 => {
            // set VX to NN
            chip8.V[nib_1 as usize] = ((nib_2 << 4) + nib_3) as u8;
        },
        0x7 => {
            // add NN to VX
            chip8.V[nib_1 as usize] += ((nib_2 << 4) + nib_3) as u8;
        },
        0x8 => {
            match nib_3 {
                0x0 => {
                    // set VX to VY
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize]
                },
                0x1 => {
                    // set VX to VX | VY
                    chip8.V[nib_1 as usize] |= chip8.V[nib_2 as usize];
                },
                0x2 => {
                    // set VX to VX & VY
                    chip8.V[nib_1 as usize] &= chip8.V[nib_2 as usize];
                },
                0x3 => {
                    // set VX to VX | VY
                    chip8.V[nib_1 as usize] ^= chip8.V[nib_2 as usize];
                },
                0x4 => {
                    // set VX to VX + VY
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
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize] >> 1;
                },
                0x7 => {
                    // set VX to VY - VX
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
                    chip8.V[nib_1 as usize] = chip8.V[nib_2 as usize];
                    chip8.V[0xF] = (chip8.V[nib_1 as usize] > 7) as u8 & 1;
                    chip8.V[nib_1 as usize] << 1;
                },
                _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
            }
        }
        0x9 => {
            // skips if VX != VY
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
            let NN: u8 = ((nib_2 << 4) + nib_3) as u8;
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
            x = chip8.V[nib_1 as usize] & (chip8.width as u8 - 1);
            y = chip8.V[nib_2 as usize] & (chip8.height as u8 - 1);
            chip8.V[0xF] = 0;
            set_bitmap(chip8, x, y, nib_3);
            sdl_draw(chip8, renderer);
        },
        0xE => {
            // TODO: key stuff
            match nib_2 {
                0x9 => {
                    match nib_3 {
                        0xE => {
                            // Checks if key is held down, PC += 2
                            match sdl.poll_events() {
                                Some((event, _timestamp)) => {
                                    match event {
                                        Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                                            chip8.pc += 2;
                                        }
                                        _ => (),
                                    }
                                }
                                None => (),
                            }
                        }
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                }
                0xA => {
                    match nib_3 {
                        1 => {
                            // TODO: this
                            // Checks if key is held down, PC += 2 if not same in vx or if not held down
                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                }
                _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
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
                            loop {
                                match sdl.poll_events() {
                                    Some((event, _timestamp)) => {
                                        match event {
                                            Event::Key { win_id, pressed, repeat, scancode, keycode, modifiers } => {
                                                chip8.V[nib_1 as usize] = keycode.0 as u8;
                                            }
                                            _ => {
                                                chip8.pc -= 2;
                                                break;
                                            },
                                        }
                                    }
                                    None => (),
                                }
                            }

                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
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
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                },
                2 => {
                    match nib_3 {
                        9 => {
                            // I set to addr of char in VX (last nibble)
                            let second_nib = chip8.V[nib_1 as usize] >> 4;
                            chip8.I = second_nib as u16;
                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                },
                3 => {
                    match nib_3 {
                        3 => {
                            let mut num = chip8.V[nib_3 as usize];
                            let mut i = 0;
                            while num > 0 {
                                chip8.mem[(chip8.I + i) as usize] = num % 10;
                                num /= 10;
                                i += 1;
                            }
                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                },
                // uses modern version
                5 => {
                    match nib_3 {
                        5 => {
                            let mut i = 0;
                            while i <= nib_1 {
                                chip8.mem[(chip8.I + i) as usize] = chip8.V[i as usize];
                                i += 1;
                            }
                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                },
                6 => {
                    match nib_3 {
                        6 => {
                            let mut i = 0;
                            while i <= nib_1 {
                                chip8.V[i as usize] = chip8.mem[(chip8.I + i) as usize];
                                i += 1;
                            }
                        },
                        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
                    }
                }
                _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
            }
        }
        _ => exit_with_error(chip8, Error::UnknownInstruction, instruct),
    }
}

fn set_bitmap(chip8: &mut Chip8, mut x: u8, mut y: u8, n: u16) {
    let mut current_row: i32 = 0;
    let mut current_byte: i32 = 0;
    let max_row_bytes = chip8.width/8;
    let x_cpy = x;
    let y_cpy = y;
    for i in 0..n {
        let byte = chip8.mem[(chip8.I + i) as usize];
        for j in 0..8 {
            let bit = (byte >> j) & 1;
            println!("({}, {})", x, y);
            if chip8.bitmap[y as usize][x as usize] == 1 && bit == 1 {
                chip8.V[0xF] = 1;
            }
            chip8.bitmap[y as usize][x as usize] |= bit;
            x += 1;
        }
        x = x_cpy;
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

    // Load in code
    chip8.pc = 0x200;
    let mut args: Vec<_> = std::env::args().collect();
    if args.len() < 2 {
        exit_with_error(&chip8, Error::NoFileGiven, 0);
    }

    let path = args.get(1).unwrap();
    let file_result = std::fs::File::open(path);
    match file_result {
        Err(_) => {
            exit_with_error(&chip8, Error::FileNotFound, 0)
        }
        Ok(_) => {}
    }

    let data: Vec<u8> = std::fs::read(path).unwrap();
    for byte in data {
        chip8.mem[chip8.pc as usize] = byte;
        chip8.pc += 1;
    }
    chip8.pc = 0x200;

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

    let mut time_start = std::time::Instant::now();
    'main_loop: loop {

        // Test Renderer
        // chip8.bitmap = [[0; 64]; 32];
        // let mut rng = rand::thread_rng();
        // for i in 0..chip8.bitmap.len() {
        //     for j in 0..chip8.bitmap[i].len() {
        //         let random_number: u8 = rng.gen_range(0..2);
        //         println!("Rand num: {random_number}");
        //         chip8.bitmap[i][j] = random_number;
        //     }
        // }
        // sdl_draw(&chip8, &renderer);


        // Timers
        let time_elapsed = time_start.elapsed().as_secs();
        if time_elapsed == 1 {
            chip8.timer_delay -= 1;
            chip8.timer_sound -= 1;
            time_start = std::time::Instant::now();

            if chip8.timer_delay == 0 {
                println!("timer delay done");
                chip8.timer_delay = 60;
            }
            if chip8.timer_sound == 0 {
                println!("timer delay done");
                chip8.timer_sound = 60;
            } else {
                beep()
            }
        }

        while let Some((event, _timestamp)) = sdl.poll_events() {

            let instruct: u16 = ((chip8.mem[chip8.pc as usize] as u16) << 8) + chip8.mem[(chip8.pc as usize)+1] as u16;
            chip8.pc += 2;
            // TODO: fix this maybe
            if chip8.pc == 4096 {
                loop{}
            }
            match event {
                Event::Quit => {
                    break 'main_loop
                },
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
                _ => execute_instruction(&mut chip8, &mut renderer, &sdl, instruct),  // TODO: fix timing, run at 60fps,
                // _ => ()
            }
        }
    }
}
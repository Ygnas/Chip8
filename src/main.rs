use std::{env, thread, time, time::Duration};
use std::{fs::File, io::Read};
use rand::prelude::*;

extern crate minifb;
mod display;
use minifb::{Key, Window, WindowOptions};
use display::*;
const WIDTH: usize = 640;
const HEIGHT: usize = 320;

struct Cpu {
    opcode: u16,
    memory: [u8; 4096],
    v_registers: [u8; 16],
    i_register: usize,
    delay: u8,
    sound_timer: u8,
    pc: usize,
    sp: usize,
    stack: [usize; 16],
    display: Display,
    b : Vec<u32>,
}

static fontset: [u8; 80] = [0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70,
0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0,
0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0, 0x10, 0xF0,
0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0,
0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0, 0x90, 0xE0, 0x90, 0xE0,
0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80];

impl Cpu {
    fn new() -> Cpu {
        let mut cpu = Cpu {
            opcode: 0,
            memory: [0; 4096],
            v_registers: [0; 16],
            i_register: 0,
            delay: 0,
            sound_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            display: Display::new(),
            b: vec![0; 64*32],
        };
        for i in 0..80 { cpu.memory[i] = fontset[i]; }
        cpu
    }

    fn load_rom(&mut self) {
        let mut f = File::open("data/test.ch8").expect("file not found");
        //let mut f = File::open(args().last().unwrap()).expect("file not found");
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).unwrap();
        for i in 0..buffer.len() {
            self.memory[0x200 + i] = buffer[i];
        }
    }

    fn read_instruction(&mut self) {
        let ten_millis = time::Duration::from_millis(1);
        //thread::sleep(ten_millis);
        self.opcode = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1]as u16);
    }

    fn execute_instruction(&mut self) {
        match self.opcode & 0xF000 {
            0x0000=> self.op_0xxx(),
            0x1000 => self.op_1xxx(),
            0x2000 => self.op_2xxx(),
            0x3000 => self.op_3xxx(),
            0x4000 => self.op_4xxx(),
            0x5000 => self.op_5xxx(),
            0x6000 => self.op_6xxx(),
            0x7000 => self.op_7xxx(),
            0x8000 => self.op_8xxx(),
            0x9000 => self.op_9xxx(),
            0xA000 => self.op_axxx(),
            0xB000 => self.op_bxxx(),
            0xC000 => self.op_cxxx(),
            0xD000 => self.op_dxxx(),
            0xE000 => self.op_exxx(),
            0xF000 => self.op_fxxx(),
            _ => print!("None {}\n",self.opcode),
        }
    }

    fn op_0xxx(&mut self) {
        match self.opcode & 0x000F{
            0x0000 => {
                //self.pc += 2;
            }
            0x000E => {
                self.pc = self.stack[self.sp];
                self.sp -= 1;
            }
            _ => print!("2 None {}\n", self.opcode),
        }
        self.pc += 2;
    }

    fn op_1xxx(&mut self) {
        self.pc = self.nnn();
    }

    fn op_2xxx(&mut self) {
        self.sp += 1;
        self.stack[self.sp] = self.pc;
        self.pc = self.nnn();
    }

    fn op_3xxx(&mut self) {
        if self.v_registers[self.x()] == self.kk() {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_4xxx(&mut self) {
        if self.v_registers[self.x()] !=self.kk(){
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_5xxx(&mut self) {
        if self.v_registers[self.x()] == self.v_registers[self.y()]{
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_6xxx(&mut self) {
        self.v_registers[self.x()] = self.kk();
        self.pc += 2;
    }

    fn op_7xxx(&mut self) {
        self.v_registers[self.x()] += self.kk();
        self.pc += 2;
    }

    fn op_8xxx(&mut self) {
        match self.opcode & 0x0F {
            0 => {
                self.v_registers[self.x()] = self.v_registers[self.y()];
                self.pc += 2;
            },
            1 => {
                self.v_registers[self.x()] = self.v_registers[self.x()] | self.v_registers[self.y()];
                self.pc += 2;
            },
            2 => {
                self.v_registers[self.x()] = self.v_registers[self.x()] & self.v_registers[self.y()];
                self.pc += 2;
            },
            3 => {
                self.v_registers[self.x()] = self.v_registers[self.x()] ^ self.v_registers[self.y()];
                self.pc += 2;
            },
            4 => {
                self.v_registers[self.x()] += self.v_registers[self.y()];
                self.v_registers[15] = if self.v_registers[self.x()] as u8 > 0xFF as u8 {1} else {0};
                self.pc += 2;
            },
            5 => {

                self.pc += 2;
            },
            6 => {
                
                self.pc += 2;
            },
            7 => {
                
                self.pc += 2;
            },
            0xE => {
                self.v_registers[15] = self.v_registers[self.x()] >> 7;
                self.v_registers[self.x()] <<= 1;
                self.pc += 2;
            },
            _ => print!("8 None")
        }
    }

    fn op_9xxx(&mut self) {
        if self.v_registers[self.x()] != self.v_registers[self.y()] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn op_axxx(&mut self) {
        self.i_register = self.nnn();
        self.pc += 2;
    }

    fn op_bxxx(&mut self) {
        self.pc = self.nnn() + self.v_registers[0] as usize;
    }

    fn op_cxxx(&mut self) {
        self.v_registers[self.x()] = rand::random::<u8>() + self.kk();
        self.pc += 2;
    }

    //Make Display!!!
    fn op_dxxx(&mut self) {
        let from = self.i_register;
        let to = from + self.n();
        let x = self.v_registers[self.x()];
        let y = self.v_registers[self.y()];
        self.v_registers[15] = self.display.draw(x as usize, y as usize, &self.memory[from..to]);
        self.pc += 2;
    }

    fn op_exxx(&mut self) {
        match self.opcode & 0x00FF {
            0x9E => {
                self.pc += 2;
            },
            0xA1 => {
                self.pc += 4;
            },
            _ => print!("E None"), 
        }
    }

    fn op_fxxx(&mut self) {
        match self.opcode & 0x00FF{
            0x55 => {
                for i in self.v_registers[0]..2 {
                    self.memory[self.i_register + i as usize] = self.v_registers[i as usize]
                }
            },
            _ => self.opcode +=1,
        }
        self.pc += 2;
    }

    fn nnn(&mut self) -> usize{
        (self.opcode & 0x0FFF) as usize
    }

    fn n(&mut self) -> usize{
        (self.opcode & 0x000F) as usize
    }

    fn x(&mut self) -> usize{
        (self.opcode & 0x0F00) as usize >> 8
    }

    fn y(&mut self) -> usize{
        (self.opcode & 0x00F0) as usize >> 4
    }

    fn kk(&mut self) -> u8{
        (self.opcode & 0x00FF) as u8
    }

    pub fn emulate(&mut self) {
        while self.display.is_open() {
            self.display.as_mut().update();

            self.read_instruction();
            self.execute_instruction();

        }
    }

}

fn main() {
    let mut cpu = Cpu::new();
    cpu.load_rom();
    //let mut buff: Vec<u32> = vec![0; WIDTH * HEIGHT];
    // let mut window = Window::new(
    //     "Chip8 Test",
    //     WIDTH,
    //     HEIGHT,
    //     WindowOptions::default(),
    // )
    // .unwrap_or_else(|e| {
    //     panic!("{}", e);
    // });
    cpu.emulate();

        //buff.append(&mut cpu.b);


        print!(" {:x}", cpu.opcode);
        print!("\n")
    
}

use crate::output::ProcessorState;
use crate::font::FONT_SET;

pub struct Processor {
    
    /// The chip-8 memory. 4096 bytes in size which means it can store 32768 bits of data
    pub memory: [u8; 4096],

    /// The registers of the chip-8 vm. 1 byte in size and there's 16 of them from V0 to VF
    pub registers: [u8; 16],

    /// The stack of chip-8. Stores return addresses when a subroutine is called
    pub stack: [usize; 48],

    /// The stack pointer. Points to the addr of the last routine
    pub sp: usize,

    /// Delay timer of chip-8. Counts down at 60Hz
    pub delay_timer: u8,

    /// Sound timer of chip-8. Counts down at 60Hz and makes buzzer sound until the value is zero
    pub sound_timer: u8,

    /// The vram of chip-8. Contains sprites to display in a 1 byte array with capacity to store 2048 values which represent the 64*32 sized display
    pub vram: [[u8; 64]; 32],

    /// Waits for keypress when EXA1 opcode is found. Indicates if the vm is actually waiting for a keypress
    pub keypresswait: bool,

    /// The key the vm is waiting for. Stored in Vx
    pub key: usize,

    /// The whole keypad
    pub keypad: [bool; 16],

    /// Program counter
    pub pc: usize,

    /// Index register pointing to a memory address
    pub i: usize,

    /// Set if any pixel is unset from set. Possible use is collision detection
    pub vram_changed: bool
}

impl Processor {
    pub fn new() -> Processor {
        let mut mem: [u8; 4096] = [0; 4096];
        for x in 0..FONT_SET.len() {
            mem[x] = FONT_SET[x];
        }

        Processor {
            memory: mem,
            registers: [0; 16],
            stack: [0; 48],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            vram: [[0; 64]; 32],
            keypresswait: false,
            key: 0,
            pc: 0x200,
            i: 0,
            vram_changed: false,
            keypad: [false; 16]
        }
    }

    pub fn tick(&mut self, keypad: [bool; 16]) -> ProcessorState {
        self.keypad = keypad;
        self.vram_changed = false;

        if self.keypresswait {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypresswait = false;
                    self.registers[self.key] = i as u8;
                    break;
                }
            }
        } else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1;
            }
            let opcode = self.get_opcode();
            self.execute_once(opcode);
        }

        ProcessorState {
            vram: self.vram.clone(),
            vram_changed: self.vram_changed,
            beep: self.sound_timer > 0
        }
    }

    pub fn load_program(&mut self, bytes: Vec<u8>) {
        for i in 0..bytes.len() {
            self.memory[i + 0x200] = bytes[i];
        }
    }

    fn get_opcode(&self) -> u16 {
        let b = (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16);
        (self.memory[self.pc] as u16) << 8 | (self.memory[self.pc + 1] as u16)
    }

    /// Executes one opcode and sets the program counter :)
    ///
    /// I yanked some code from https://github.com/starrhorne/chip8-rust/blob/master/src/processor.rs as I'm noob
    fn execute_once(&mut self, opcode: u16) {
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        // Super chip-8 ins
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op00ee(),
            (0x01, _, _, _) => self.op1nnn(nnn),
            (0x02, _, _, _) => self.op2nnn(nnn),
            (0x03, _, _, _) => self.op3xkk(x, kk),
            (0x04, _, _, _) => self.op4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op5xy0(x, y),
            (0x06, _, _, _) => self.op6xkk(x, kk),
            (0x07, _, _, _) => self.op7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op8xy0(x, y),
            (0x08, _, _, 0x01) => self.op8xy1(x, y),
            (0x08, _, _, 0x02) => self.op8xy2(x, y),
            (0x08, _, _, 0x03) => self.op8xy3(x, y),
            (0x08, _, _, 0x04) => self.op8xy4(x, y),
            (0x08, _, _, 0x05) => self.op8xy5(x, y),
            (0x08, _, _, 0x06) => self.op8x06(x),
            (0x08, _, _, 0x07) => self.op8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op8x0e(x),
            (0x09, _, _, 0x00) => self.op9xy0(x, y),
            (0x0a, _, _, _) => self.opannn(nnn),
            (0x0b, _, _, _) => self.opbnnn(nnn),
            (0x0c, _, _, _) => self.opcxkk(x, kk),
            (0x0d, _, _, _) => self.opdxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.opex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.opexa1(x),
            (0x0f, _, 0x00, 0x07) => self.opfx07(x),
            (0x0f, _, 0x00, 0x0a) => self.opfx0a(x),
            (0x0f, _, 0x01, 0x05) => self.opfx15(x),
            (0x0f, _, 0x01, 0x08) => self.opfx18(x),
            (0x0f, _, 0x01, 0x0e) => self.opfx1e(x),
            (0x0f, _, 0x02, 0x09) => self.opfx29(x),
            (0x0f, _, 0x03, 0x03) => self.opfx33(x),
            (0x0f, _, 0x05, 0x05) => self.opfx55(x),
            (0x0f, _, 0x06, 0x05) => self.opfx65(x),
            _ => self.pc_next()
        }
    }

    /// Clears the vram
    fn op00e0(&mut self) {
        for x in 0..32 {
            for y in 0..64 {
                self.vram[x][y] = 0;
            }
        }

        self.vram_changed = true;
        self.pc_next();
    }

    fn op00ee(&mut self) {
        dbg!("op00ee");
        self.sp -= 1;
        self.pc_jump(self.stack[self.sp]);
    }

    fn op1nnn(&mut self, nnn: usize) {
        dbg!("op1nnn");
        dbg!(nnn);
        self.pc_jump(nnn);
    }

    fn op2nnn(&mut self, nnn: usize) {
        dbg!("op2nnn");
        dbg!(nnn);
        self.stack[self.sp] = self.pc + 2; // Next opcode
        self.sp += 1;
        self.pc_jump(nnn);
    }

    fn op3xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] == kk {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn op4xkk(&mut self, x: usize, kk: u8) {
        if self.registers[x] != kk {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn op5xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn op6xkk(&mut self, x: usize, kk: u8) {
        self.registers[x] = kk;
        self.pc_next();
    }

    fn op7xkk(&mut self, x: usize, kk: u8) {
        let (sum, ovrflw) = self.registers[x].overflowing_add(kk);
        self.registers[x] = sum;
        self.registers[0x0f] = ovrflw as u8;
        self.pc_next();
    }

    fn op8xy0(&mut self, x: usize, y: usize) {
        self.registers[x] = self.registers[y];
        self.pc_next();
    }

    fn op8xy1(&mut self, x: usize, y: usize) {
        self.registers[x] |= self.registers[y];
        self.pc_next();
    }

    fn op8xy2(&mut self, x: usize, y: usize) {
        self.registers[x] &= self.registers[y];
        self.pc_next(); 
    }

    fn op8xy3(&mut self, x: usize, y: usize) {
        self.registers[x] ^= self.registers[y];
        self.pc_next();
    }

    fn op8xy4(&mut self, x: usize, y: usize) {
        let vx = self.registers[x] as u16;
        let vy = self.registers[y] as u16;
        let result = vx + vy;

        self.registers[x] = result as u8;
        self.registers[0x0f] = if result > 0xff { 1 } else { 0 };
        self.pc_next();
    }

    fn op8xy5(&mut self, x: usize, y: usize) {
        self.registers[0x0f] = if self.registers[x] > self.registers[y] { 1 } else { 0 };
        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]);
        self.pc_next();
    }

    fn op8x06(&mut self, x: usize) {
        self.registers[0x0f] = self.registers[x] & 1;
        self.registers[x] >>= 1;
        self.pc_next();
    }

    fn op8xy7(&mut self, x: usize, y: usize) {
        self.registers[0x0f] = if self.registers[y] > self.registers[x] { 1 } else { 0 };
        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]);
        self.pc_next();
    }

    fn op8x0e(&mut self, x: usize) {
        self.registers[0x0f] = (self.registers[x] & 0b10000000) >> 7;
        self.registers[x] <<= 1;
    }

    fn op9xy0(&mut self, x: usize, y: usize) {
        if self.registers[x] != self.registers[y] {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn opannn(&mut self, nnn: usize) {
        self.i = nnn;
        self.pc_next();
    }

    fn opbnnn(&mut self, nnn: usize) {
        dbg!("opbnnn");
        dbg!(nnn);
        self.pc_jump((self.registers[0] as usize) + nnn);
    }

    fn opcxkk(&mut self, x: usize, kk: u8) {
        let mut rng = rand::thread_rng();
        self.registers[x] = rand::Rng::gen::<u8>(&mut rng) & kk;
        self.pc_next();
    }

    fn opdxyn(&mut self, x: usize, y: usize, n: usize) {
        // ...
        // I don't know what I'm doing -_-
        // yanked directly from https://github.com/starrhorne/chip8-rust/blob/345602a97288fd8d69dafd6684e8f51cd38e95e2/src/processor.rs#L340

        self.registers[0x0f] = 0;
        for byte in 0..n {
            let y = (self.registers[y] as usize + byte) % 32;
            for bit in 0..8 {
                let x = (self.registers[x] as usize + bit) % 64;
                let color = (self.memory[self.i + byte] >> (7 - bit)) & 1;
                self.registers[0x0f] |= color & self.vram[y][x];
                self.vram[y][x] ^= color;

            }
        }
        self.vram_changed = true;
        self.pc_next();
    }
    
    fn opex9e(&mut self, x: usize) {
        if self.keypad[self.registers[x] as usize] {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn opexa1(&mut self, x: usize) {
        if !self.keypad[self.registers[x] as usize] {
            self.pc_skip();
        }
        else {
            self.pc_next();
        }
    }

    fn opfx07(&mut self, x: usize) {
        self.registers[x] = self.delay_timer;
        self.pc_next();
    }

    fn opfx0a(&mut self, x: usize) {
        self.keypresswait = true;
        self.key = x;
        self.pc_next();
    }

    fn opfx15(&mut self, x: usize) {
        self.delay_timer = self.registers[x];
        self.pc_next();
    }

    fn opfx18(&mut self, x: usize) {
        self.sound_timer = self.registers[x];
        self.pc_next();
    }

    fn opfx1e(&mut self, x: usize) {
        self.i += self.registers[x] as usize;
        self.registers[0x0f] = if self.i > 0x0F00 { 1 } else { 0 };
        self.pc_next();
    }

    fn opfx29(&mut self, x: usize) {
        self.i = (self.registers[x] as usize) * 5;
        self.pc_next();
    }

    fn opfx33(&mut self, x: usize) {
        self.memory[self.i] = self.registers[x] / 100;
        self.memory[self.i + 1] = (self.registers[x] % 100) / 10;
        self.memory[self.i + 2] = self.registers[x] % 10;
        self.pc_next();
    }

    fn opfx55(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.memory[self.i + i] = self.registers[i];
        }
        self.pc_next();
    }

    fn opfx65(&mut self, x: usize) {
        for i in 0..x + 1 {
            self.registers[i] = self.memory[self.i + i];
        }
        self.pc_next();
    }

    fn pc_next(&mut self) {
        self.pc += 2;
    }

    fn pc_jump(&mut self, addr: usize) {
        dbg!(addr);
        self.pc = addr;
    }

    fn pc_skip(&mut self) {
        dbg!(self.pc);
        self.pc += 4;
    }
}
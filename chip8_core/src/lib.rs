use rand::random;

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const START_ADDR: u16 = 0x200;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
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

pub struct Chip8 {
    pc: u16,                     // Program Counter
    ram: [u8; RAM_SIZE],         // RAM
    screen: [bool; SCREEN_SIZE], // Screen
    v_regs: [u8; NUM_REGS],      // V Registers
    i_reg: u16,                  // Index Register(Singular!)
    stack: [u16; STACK_SIZE],    // The Stack
    sp: u16,                     // Stack Pointer
    keys: [bool; NUM_KEYS],      // Keys
    dt: u8,                      // Delay Timer
    st: u8,                      // Sound Timer
}

impl Default for Chip8 {
    fn default() -> Self {
        let mut chip8 = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_SIZE],
            v_regs: [0; NUM_REGS],
            i_reg: 0,
            stack: [0; STACK_SIZE],
            sp: 0,
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        chip8.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        chip8
    }
}

impl Chip8 {
    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_SIZE];
        self.v_regs = [0; NUM_REGS];
        self.i_reg = 0;
        self.stack = [0; STACK_SIZE];
        self.sp = 0;
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        // Check if the stack is empty before poping
        assert!(self.sp != 0);
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timer(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEEEEEP
            }
            self.st -= 1;
        }
    }

    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();
        self.ram[start..end].copy_from_slice(data);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            // NOP
            (0, 0, 0, 0) => (),
            // CLS
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_SIZE],
            // RET
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            }
            // JMP NNN
            (1, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = nnn;
            }
            // CALL NNN
            (2, _, _, _) => {
                let nnn = op & 0xFFF;
                self.push(self.pc);
                self.pc = nnn;
            }
            // SKIP VX == NN
            (3, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] == nn {
                    self.pc += 2;
                }
            }
            // SKIP VX != NN
            (4, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_regs[x] != nn {
                    self.pc += 2;
                }
            }
            // SKIP VX == VY
            (5, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] == self.v_regs[y] {
                    self.pc += 2;
                }
            }
            // VX = NN
            (6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = nn;
            }
            // VX += NN
            (7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_regs[x] = self.v_regs[x].wrapping_add(nn);
            }
            // VX = VY
            (8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] = self.v_regs[y];
            }
            // VX |= VY
            (8, _, _, 1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] |= self.v_regs[y];
            }
            // VX |= VY
            (8, _, _, 2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] &= self.v_regs[y];
            }
            // VX |= VY
            (8, _, _, 3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_regs[x] ^= self.v_regs[y];
            }
            // VX += VY
            (8, _, _, 4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_regs[x].overflowing_add(self.v_regs[y]);
                let new_vf = u8::from(carry);

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            // VX -= VY
            (8, _, _, 5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[x].overflowing_sub(self.v_regs[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            // VX >>= 1
            (8, _, _, 6) => {
                let x = digit2 as usize;
                let lsb = self.v_regs[x] & 1;
                self.v_regs[x] >>= 1;
                self.v_regs[0xF] = lsb;
            }
            // VX = VY - VX
            (8, _, _, 7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_regs[y].overflowing_sub(self.v_regs[x]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_regs[x] = new_vx;
                self.v_regs[0xF] = new_vf;
            }
            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = digit2 as usize;
                let msb = (self.v_regs[x] >> 7) & 1;
                self.v_regs[x] <<= 1;
                self.v_regs[0xF] = msb;
            }
            // SKIP VX != VY
            (9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_regs[x] != self.v_regs[y] {
                    self.pc += 2;
                }
            }
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            // JMP V0 + NNN
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_regs[0] as u16) + nnn;
            }
            // VX = rand() & NN
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();
                self.v_regs[x] = rng & nn;
            }
            // DRAW
            (0xD, _, _, _) => {
                let x_coord = self.v_regs[digit2 as usize] as u16;
                let y_coord = self.v_regs[digit3 as usize] as u16;
                let num_rows = digit4;

                let mut flipped = false;
                for y_line in 0..num_rows {
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    for x_line in 0..8 {
                        if (pixels & (0b1000_0000 >> x_line)) != 0 {
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;

                            let idx = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }
                if flipped {
                    self.v_regs[0xF] = 1;
                } else {
                    self.v_regs[0xF] = 0;
                }
            }
            // SKIP KEY PRESS
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let key = self.keys[self.v_regs[x] as usize];
                if key {
                    self.pc += 2;
                }
            }
            // SKIP KEY RELEASE
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let key = self.keys[self.v_regs[x] as usize];
                if !key {
                    self.pc += 2;
                }
            }
            // VX = DT
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_regs[x] = self.dt;
            }
            // WAIT KEY
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_regs[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    // Redo OpCode
                    self.pc -= 2;
                }
            }
            // DT = VX
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_regs[x];
            }
            // ST = VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_regs[x];
            }
            // I += VX
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            }
            // I = FONT
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_regs[x] as u16;
                self.i_reg = c * 5;
            }
            // BCD
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_regs[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            }
            // STORE V0 - VX
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.ram[i + idx] = self.v_regs[idx];
                }
            }
            // LOAD V0 - VX
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;
                for idx in 0..=x {
                    self.v_regs[idx] = self.ram[i + idx];
                }
            }
            (_, _, _, _) => unimplemented!("Opcode {} not implemented", op),
        }
    }
}

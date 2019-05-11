use crate::instruction::Instruction;
use crate::screen::{Screen, SPRITES};

enum LoadKeyState {
    None,
    WaitPress { reg: usize },
    WaitRelease { reg: usize, key: usize },
}

pub struct Machine {
    freq_multiplier: u8,
    timer_decrease: u8,

    memory: [u8; 4096],
    memory_pos: usize,

    stack: [usize; 16],
    stack_pos: usize,

    registers: [u8; 16],
    i: usize,
    delay: u8,
    sound: u8,

    load_key: LoadKeyState,

    screen: Screen,
}

impl Machine {
    pub fn new(freq_multiplier: u8, program: &[u8]) -> Self {
        let mut memory = [0; 4096];
        memory[0..SPRITES.len()].copy_from_slice(&SPRITES);

        memory[0x200..0x200 + program.len()].copy_from_slice(program);

        Self {
            freq_multiplier,
            timer_decrease: 0,

            memory,
            memory_pos: 0x200,

            stack: [0; 16],
            stack_pos: 0,

            registers: [0; 16],
            i: 0,
            delay: 0,
            sound: 0,

            load_key: LoadKeyState::None,

            screen: Screen::new(),
        }
    }

    pub fn open(freq_multiplier: u8, path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        use std::io::Read;

        let mut file = std::fs::File::open(path)?;
        let mut program = Vec::new();
        file.read_to_end(&mut program)?;

        Ok(Self::new(freq_multiplier, &program))
    }

    pub fn step(&mut self, pressed_keys: [bool; 16]) {
        match self.load_key {
            LoadKeyState::None => {
                if self.timer_decrease == 0 {
                    if self.delay != 0 {
                        self.delay -= 1;
                    }
                    if self.sound != 0 {
                        self.sound -= 1;
                    }
                    self.timer_decrease = self.freq_multiplier;
                }
                self.timer_decrease -= 1;

                let instr = Instruction::parse(
                    self.memory[self.memory_pos],
                    self.memory[self.memory_pos + 1],
                )
                .unwrap();

                println!(
                    "0x{:03X}: ({:02X}{:02X}) {}",
                    self.memory_pos,
                    self.memory[self.memory_pos],
                    self.memory[self.memory_pos + 1],
                    instr
                );
                self.execute_instruction(instr, pressed_keys);
            },
            LoadKeyState::WaitPress { reg } => {
                for (i, key) in pressed_keys.iter().enumerate() {
                    if *key {
                        self.load_key = LoadKeyState::WaitRelease { reg, key: i };
                        break;
                    }
                }
            },
            LoadKeyState::WaitRelease { reg, key } => {
                if !pressed_keys[key] {
                    self.load_key = LoadKeyState::None;
                    self.registers[reg] = key as _;
                }
            },
        }
    }

    fn execute_instruction(&mut self, instr: Instruction, pressed_keys: [bool; 16]) {
        let mut increase_mem_pos = 2;

        match instr {
            Instruction::Jump(a) => {
                // Jump to a
                self.memory_pos = a;
                increase_mem_pos = 0;
            },
            Instruction::JumpPlus(a) => {
                // Jump to V0 + a
                self.memory_pos = self.registers[0] as usize + a;
                increase_mem_pos = 0;
            },
            Instruction::Call(a) => {
                // Call procedure at a
                self.stack[self.stack_pos] = self.memory_pos + 2;
                self.stack_pos += 1;
                self.memory_pos = a;
                increase_mem_pos = 0;
            },
            Instruction::Return => {
                // Return from procedure
                self.stack_pos -= 1;
                self.memory_pos = self.stack[self.stack_pos];
                increase_mem_pos = 0;
            },

            Instruction::SkipEqByte(x, b) => {
                // Skip instruction if Vx == b
                if self.registers[x] == b {
                    increase_mem_pos += 2;
                }
            },
            Instruction::SkipNeByte(x, b) => {
                // Skip instruction if Vx != b
                if self.registers[x] != b {
                    increase_mem_pos += 2;
                }
            },
            Instruction::SkipEq(x, y) => {
                // Skip instruction if Vx == Vy
                if self.registers[x] == self.registers[y] {
                    increase_mem_pos += 2;
                }
            },
            Instruction::SkipNe(x, y) => {
                // Skip instruction if Vx != Vy
                if self.registers[x] != self.registers[y] {
                    increase_mem_pos += 2;
                }
            },
            Instruction::SkipPressed(x) => {
                // Skip instruction if Vx == pressed key
                if pressed_keys[self.registers[x] as usize] {
                    increase_mem_pos += 2;
                }
            },
            Instruction::SkipNPressed(x) => {
                // Skip instruction if Vx != pressed key
                if !pressed_keys[self.registers[x] as usize] {
                    increase_mem_pos += 2;
                }
            },

            Instruction::LoadByte(x, b) => {
                // Vx = b
                self.registers[x] = b;
            },
            Instruction::LoadRandom(x, b) => {
                // Vx = random() & b
                self.registers[x] = rand::random::<u8>() & b;
            },
            Instruction::AddByte(x, b) => {
                // Vx = Vx + b
                self.registers[x] = self.registers[x].wrapping_add(b);
            },
            Instruction::Copy(x, y) => {
                // Vx = Vy
                self.registers[x] = self.registers[y];
            },
            Instruction::Or(x, y) => {
                // Vx = Vx | Vy
                self.registers[x] |= self.registers[y];
            },
            Instruction::And(x, y) => {
                // Vx = Vx & Vy
                self.registers[x] &= self.registers[y];
            },
            Instruction::Xor(x, y) => {
                // Vx = Vx ^ Vy
                self.registers[x] ^= self.registers[y];
            },
            Instruction::Add(x, y) => {
                // Vx = Vx + Vy, VF = 1 of overflowed
                let (val, ovf) = self.registers[x].overflowing_add(self.registers[y]);
                self.registers[x] = val;
                self.registers[0xF] = ovf as u8;
            },
            Instruction::Sub(x, y) => {
                // Vx = Vx - Vy, VF = 1 if borrowed
                let (val, ovf) = self.registers[x].overflowing_sub(self.registers[y]);
                self.registers[x] = val;
                self.registers[0xF] = !ovf as u8;
            },
            Instruction::Subn(x, y) => {
                // Vx = Vy - Vx, VF = 1 if borrowed
                let (val, ovf) = self.registers[y].overflowing_sub(self.registers[x]);
                self.registers[x] = val;
                self.registers[0xF] = !ovf as u8;
            },
            Instruction::Shr(x) => {
                // Vx = Vx SHR 1, VF = rightmost bit before SHR
                self.registers[0xF] = self.registers[x] & 0x1;
                self.registers[x] >>= 1;
            },
            Instruction::Shl(x) => {
                // Vx = Vx SHL 1, VF = leftmost bit before SHL
                self.registers[0xF] = (self.registers[x] & 0x80) >> 7;
                self.registers[x] <<= 1;
            },
            Instruction::LoadDelay(x) => {
                // Vx = DT
                self.registers[x] = self.delay;
            },
            Instruction::LoadPressed(x) => {
                // Vx = released button (wait for a release)
                self.load_key = LoadKeyState::WaitPress { reg: x };
            },
            Instruction::SetDelay(x) => {
                // DT = Vx
                self.delay = self.registers[x];
            },
            Instruction::SetSound(x) => {
                // ST = Vx
                self.sound = self.registers[x];
            },

            Instruction::LoadI(a) => {
                // I = a
                self.i = a;
            },
            Instruction::AddToI(x) => {
                // I = I + x
                self.i += self.registers[x] as usize;
            },
            Instruction::SetSprite(x) => {
                // I = location of a sprite for a digit stored in Vx
                self.i = (self.registers[x] as usize % 0x10) * 5;
            },
            Instruction::StoreBCD(x) => {
                // Store BCD representation of a Vx in memory[I..I+2]
                self.memory[self.i] = self.registers[x] / 100 % 10;
                self.memory[self.i + 1] = self.registers[x] / 10 % 10;
                self.memory[self.i + 2] = self.registers[x] % 10;
            },
            Instruction::StoreRegisters(x) => {
                // Store registers[0..x] in memory[i..i+x]
                self.memory[self.i..=self.i + x].copy_from_slice(&self.registers[0..=x]);
            },
            Instruction::RestoreRegisters(x) => {
                // Restore registers from memory[i..i+x] into reisters[0..x]
                self.registers[0..=x].copy_from_slice(&self.memory[self.i..=self.i + x]);
            },

            Instruction::Clear => {
                // Clear screen
                self.screen.clear();
            },
            Instruction::Draw(x, y, n) => {
                // Draw a sprite from memory[i..i+n] at (Vx, Vy), VF - collision
                self.registers[0xF] = self.screen.draw(
                    self.registers[x],
                    self.registers[y],
                    &self.memory[self.i..self.i + n as usize],
                ) as _;
            },
        }

        self.memory_pos += increase_mem_pos;
    }

    pub fn screen(&mut self) -> &mut Screen {
        &mut self.screen
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    Jump(usize),
    JumpPlus(usize),
    Call(usize),
    Return,

    SkipEqByte(usize, u8),
    SkipNeByte(usize, u8),
    SkipEq(usize, usize),
    SkipNe(usize, usize),
    SkipPressed(usize),
    SkipNPressed(usize),

    LoadByte(usize, u8),
    LoadRandom(usize, u8),
    AddByte(usize, u8),
    Copy(usize, usize),
    Or(usize, usize),
    And(usize, usize),
    Xor(usize, usize),
    Add(usize, usize),
    Sub(usize, usize),
    Subn(usize, usize),
    Shr(usize),
    Shl(usize),
    LoadDelay(usize),
    LoadPressed(usize),
    SetDelay(usize),
    SetSound(usize),

    LoadI(usize),
    AddToI(usize),
    SetSprite(usize),
    StoreBCD(usize),
    StoreRegisters(usize),
    RestoreRegisters(usize),

    Clear,
    Draw(usize, usize, u8),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Instruction::*;
        match self {
            Jump(a) =>              write!(f, "JP   0x{:03X}", a),
            JumpPlus(a) =>          write!(f, "JP   V0, 0x{:03X}", a),
            Call(a) =>              write!(f, "CALL 0x{:03X}", a),
            Return =>               write!(f, "RET"),
            
            SkipEqByte(x, b) =>     write!(f, "SE   V{:X}, 0x{:02X}", x, b),
            SkipNeByte(x, b) =>     write!(f, "SNE  V{:X}, 0x{:02X}", x, b),
            SkipEq(x, y) =>         write!(f, "SE   V{:X}, V{:X}", x, y),
            SkipNe(x, y) =>         write!(f, "SNE  V{:X}, V{:X}", x, y),
            SkipPressed(x) =>       write!(f, "SKP  V{:X}", x),
            SkipNPressed(x) =>      write!(f, "SKNP V{:X}", x),

            LoadByte(x, b) =>       write!(f, "LD   V{:X}, 0x{:02X}", x, b),
            LoadRandom(x, b) =>     write!(f, "RND  V{:X}, 0x{:02X}", x, b),
            AddByte(x, b) =>        write!(f, "ADD  V{:X}, 0x{:02X}", x, b),
            Copy(x, y) =>           write!(f, "LD   V{:X}, V{:X}", x, y),
            Or(x, y) =>             write!(f, "OR   V{:X}, V{:X}", x, y),
            And(x, y) =>            write!(f, "AND  V{:X}, V{:X}", x, y),
            Xor(x, y) =>            write!(f, "XOR  V{:X}, V{:X}", x, y),
            Add(x, y) =>            write!(f, "ADD  V{:X}, V{:X}", x, y),
            Sub(x, y) =>            write!(f, "SUB  V{:X}, V{:X}", x, y),
            Subn(x, y) =>           write!(f, "SUBN V{:X}, V{:X}", x, y),
            Shr(x) =>               write!(f, "SHR  V{:X}", x),
            Shl(x) =>               write!(f, "SHL  V{:X}", x),
            LoadDelay(x) =>         write!(f, "LD   V{:X}, DT", x),
            LoadPressed(x) =>       write!(f, "LD   V{:X}, K", x),
            SetDelay(x) =>          write!(f, "LD   DT, V{:X}", x),
            SetSound(x) =>          write!(f, "LD   ST, V{:X}", x),

            LoadI(a) =>             write!(f, "LD   I, 0x{:03X}", a),
            AddToI(x) =>            write!(f, "ADD  I, V{:X}", x),
            SetSprite(x) =>         write!(f, "LD   F, V{:X}", x),
            StoreBCD(x) =>          write!(f, "LD   B, V{:X}", x),
            StoreRegisters(x) =>    write!(f, "LD   [I], {:X}", x),
            RestoreRegisters(x) =>  write!(f, "LD   {:X}, [I]", x),

            Clear =>                write!(f, "CLS"),
            Draw(x, y, n) =>        write!(f, "DRW  V{:X}, V{:X}, {:X}", x, y, n),
        }
    }
}

impl Instruction {
    pub fn parse(op1: u8, op2: u8) -> Option<Self> {
        use Instruction::*;

        fn addr(op1: u8, op2: u8) -> usize {
            ((op1 as usize & 0x0F) << 8) + op2 as usize
        }
        fn x(op1: u8) -> usize {
            (op1 & 0x0F) as _
        }
        fn y(op2: u8) -> usize {
            ((op2 & 0xF0) >> 4) as _
        }

        match op1 & 0xF0 {
            0x00 => match op2 {
                0xE0 => Some(Clear),
                0xEE => Some(Return),
                _ => None,
            },
            0x10 => Some(Jump(addr(op1, op2))),
            0x20 => Some(Call(addr(op1, op2))),
            0x30 => Some(SkipEqByte(x(op1), op2)),
            0x40 => Some(SkipNeByte(x(op1), op2)),
            0x50 if op2 & 0x0F == 0x0 => Some(SkipEq(x(op1), y(op2))),
            0x60 => Some(LoadByte(x(op1), op2)),
            0x70 => Some(AddByte(x(op1), op2)),
            0x80 => match op2 & 0x0F {
                0x0 => Some(Copy(x(op1), y(op2))),
                0x1 => Some(Or(x(op1), y(op2))),
                0x2 => Some(And(x(op1), y(op2))),
                0x3 => Some(Xor(x(op1), y(op2))),
                0x4 => Some(Add(x(op1), y(op2))),
                0x5 => Some(Sub(x(op1), y(op2))),
                0x6 => Some(Shr(x(op1))),
                0x7 => Some(Subn(x(op1), y(op2))),
                0xE => Some(Shl(x(op1))),
                _ => None,
            },
            0x90 => Some(SkipNe(x(op1), y(op2))),
            0xA0 => Some(LoadI(addr(op1, op2))),
            0xB0 => Some(JumpPlus(addr(op1, op2))),
            0xC0 => Some(LoadRandom(x(op1), op2)),
            0xD0 => Some(Draw(x(op1), y(op2), op2 & 0x0F)),
            0xE0 => match op2 {
                0x9E => Some(SkipPressed(x(op1))),
                0xA1 => Some(SkipNPressed(x(op1))),
                _ => None,
            },
            0xF0 => match op2 {
                0x07 => Some(LoadDelay(x(op1))),
                0x0A => Some(LoadPressed(x(op1))),
                0x15 => Some(SetDelay(x(op1))),
                0x18 => Some(SetSound(x(op1))),
                0x1E => Some(AddToI(x(op1))),
                0x29 => Some(SetSprite(x(op1))),
                0x33 => Some(StoreBCD(x(op1))),
                0x55 => Some(StoreRegisters(x(op1))),
                0x65 => Some(RestoreRegisters(x(op1))),
                _ => None,
            },
            _ => None,
        }
    }
}

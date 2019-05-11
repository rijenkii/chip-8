pub const SPRITES: [u8; 5 * 16] = [
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

pub struct Screen {
    buffer: [[bool; 64]; 32],
    redraw: bool,
}

impl Screen {
    pub fn new() -> Self {
        Self {
            buffer: [[false; 64]; 32],
            redraw: true,
        }
    }

    pub fn clear(&mut self) {
        self.buffer = [[false; 64]; 32];
        self.redraw = true;
    }

    pub fn draw(&mut self, x_start: u8, y_start: u8, sprite: &[u8]) -> bool {
        let mut collision = false;

        for y in 0..sprite.len() {
            for x in 0..8 {
                let x_pos = (x_start as usize + x) % 64;
                let y_pos = (y_start as usize + y) % 32;
                let pix = &mut self.buffer[y_pos][x_pos];

                let sprite_pix = sprite[y as usize] & (1 << (7 - x)) != 0;

                if !*pix && sprite_pix {
                    *pix = true;
                    self.redraw = true;
                } else if *pix && sprite_pix {
                    *pix = false;
                    collision = true;
                    self.redraw = true;
                }
            }
        }

        collision
    }

    pub fn buffer(&self) -> [[bool; 64]; 32] {
        self.buffer
    }
    pub fn needs_redraw(&self) -> bool {
        self.redraw
    }
    pub fn redrawn(&mut self) {
        self.redraw = false;
    }
}

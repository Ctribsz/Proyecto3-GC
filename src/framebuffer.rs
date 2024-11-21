pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub buffer1: Vec<u32>,
    pub buffer2: Vec<u32>,
    pub zbuffer: Vec<f32>,
    background_color: u32,
    current_color: u32,
    active_buffer: bool,
}

impl Framebuffer {

    pub fn draw_line(&mut self, x1: usize, y1: usize, x2: usize, y2: usize) {
        let dx = (x2 as isize - x1 as isize).abs();
        let dy = -(y2 as isize - y1 as isize).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx + dy;
    
        let mut x = x1 as isize;
        let mut y = y1 as isize;
    
        loop {
            if x >= 0 && x < self.width as isize && y >= 0 && y < self.height as isize {
                let index = y as usize * self.width + x as usize;
                self.set_color_at_index(index, self.current_color, 0.0);
            }
    
            if x == x2 as isize && y == y2 as isize {
                break;
            }
    
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_point(&mut self, x: usize, y: usize) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            println!("Set color {:X} at index {}", self.current_color, index); // Debug
            self.set_color_at_index(index, self.current_color, 0.0);
        }
    }    

    pub fn set_color_at_index(&mut self, index: usize, color: u32, depth: f32) {
        if index < self.zbuffer.len() && self.zbuffer[index] > depth {
            if color != 0x0 { // Previene el uso de un color vacío accidentalmente
                if self.active_buffer {
                    self.buffer1[index] = color;
                } else {
                    self.buffer2[index] = color;
                }
                self.zbuffer[index] = depth;
            }
        }
    }   

    pub fn new(width: usize, height: usize) -> Self {
        Framebuffer {
            width,
            height,
            buffer1: vec![0; width * height],
            buffer2: vec![0; width * height],
            zbuffer: vec![f32::INFINITY; width * height],
            background_color: 0x000000,
            current_color: 0xFFFFFF,
            active_buffer: true,
        }
    }

    pub fn clear(&mut self) {
        if self.active_buffer {
            self.buffer1.fill(self.background_color);
        } else {
            self.buffer2.fill(self.background_color);
        }
        self.zbuffer.fill(f32::INFINITY);
    }

    pub fn get_active_buffer(&self) -> &[u32] {
        if self.active_buffer {
            &self.buffer1
        } else {
            &self.buffer2
        }
    }    

    pub fn switch_buffers(&mut self) {
        self.active_buffer = !self.active_buffer;
    }

    pub fn set_background_color(&mut self, color: u32) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: u32) {
        self.current_color = color;
    }

    pub fn get_background_color(&self) -> u32 {
        self.background_color
    }

    pub fn get_current_color(&self) -> u32 {
        self.current_color
    }
}
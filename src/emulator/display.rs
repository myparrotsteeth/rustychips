pub mod display {
    pub struct Display {
        pixels: [[bool;64];32]
    }
    
    impl Display {
        pub fn new() -> Display {
            Display {
                pixels: [[false; 64];32]
            }
        }
    
        pub fn clear(&mut self) {
            self.pixels = [[false; 64];32];
        }

        pub fn write_sprite(&mut self, x:usize, y:usize, sprite_bytes: &Vec<u8>) -> bool {
            let mut collision = false;
            for (i, byte) in sprite_bytes.iter().enumerate() {
                collision |= self.write_row_buffer(x, y+i, *byte);
            }
            collision
        }

        pub fn write_row_buffer(&mut self, x:usize,y:usize, row_as_byte: u8) -> bool {
            let mut collision = false;
            for i in 0..8 {
                let bit_value = (( row_as_byte >> (7-i) ) & 1) > 0;
                //println!("i = {}, j = {}, x = {}, y = {}, row_byte = {}, bit = {}", 
                //    i, j, x_coord, y_coord, row_byte, bit_value);
                let screen_pixel = &mut (self.pixels[y][(x+i)%64]);
                if bit_value {
                    if *screen_pixel == true {
                        *screen_pixel = false;
                        collision = true;
                    }
                    else { 
                        *screen_pixel = true;
                    }
                }
            }

            collision
        }
    
        pub fn draw(&self) {
            print!("{}[2J", 27 as char);
            for row in self.pixels {
                for pixel in row {
                    if pixel {
                        print!("▉"); //("█")
                    }
                    else {
                        print!(" ");
                    }
                }
                println!("");
            }
        }
    }
}
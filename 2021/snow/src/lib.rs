#![no_std]

use core::panic::PanicInfo;

extern {
    fn js_random() -> f32;
    fn js_sin(x: f32) -> f32;
}

#[no_mangle]
static mut IMAGE_BUFFER: Framebuffer = Framebuffer([0u32; WIDTH * HEIGHT]);
static mut LOCAL_DATA: LocalData = LocalData::new();

const WIDTH: usize = 600;
const HEIGHT: usize = 800;

const COLOUR_FLAKE: u32 = 0xFF_FF_FF_FF; // RGBA
const COLOUR_BACKGROUND: u32 = 0x00_00_00_00; // RGBA

const FLAKE_COUNT: usize = 10240;
const RESPAWN_HEIGHT_JITTER: f32 = 5.0;
const TERMINAL_VELOCITY: f32 = 0.2;
const SPIN_RADIUS_LOW: f32 = 0.2;
const SPIN_RADIUS_HIGH: f32 = 0.5;
const SPIN_SPEED: usize = 1;

type SineTable = [f32; 628];

struct Framebuffer([u32; WIDTH * HEIGHT]);

impl Framebuffer {
    fn get(&self, x: usize, y: usize) -> Result<u32, ()> {
        let index = x + y * WIDTH;

        if index < self.0.len() {
            Ok(self.0[index])
        } else {
            Err(())
        }
    }

    fn set(&mut self, x: usize, y: usize, value: u32) -> Result<(), ()> {
        let index = x + y * WIDTH;

        if index < self.0.len() {
            self.0[index] = value;
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Copy, Clone)]
struct Snowflake {
    x: f32,
    y: f32,
    seed: usize,
    radius: f32,
}

impl Snowflake {
    const fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            seed: 0,
            radius: 0.0,
        }
    }

    fn randomize(&mut self) {
        self.x = random() * WIDTH as f32;
        self.y = random() * HEIGHT as f32;
        self.seed = (random() * 628.0) as usize;

        self.radius = (random() * (SPIN_RADIUS_HIGH - SPIN_RADIUS_LOW)) + SPIN_RADIUS_LOW;
    }

    /// Update position of a single snowflake
    fn animate(&mut self, sine_lookup: &SineTable) {
        self.y += TERMINAL_VELOCITY;
        self.seed = (self.seed + SPIN_SPEED) % 628;

        self.x += sine_lookup[self.seed] * self.radius;
    }
}

#[no_mangle]
#[allow(static_mut_refs)]
pub unsafe extern fn init() {
    let local_data = &mut LOCAL_DATA;

    local_data.fill_snowbank();
    local_data.randomize_snowflakes();
    local_data.calculate_sine_table();
}

#[no_mangle]
#[allow(static_mut_refs)]
pub unsafe extern fn render() {
    LOCAL_DATA.clear_snowflakes(&mut IMAGE_BUFFER);
    LOCAL_DATA.draw_snowflakes(&mut IMAGE_BUFFER);
}

fn random() -> f32 {
    unsafe { js_random() }
}

fn sin(x: f32) -> f32 {
    unsafe { js_sin(x) }
}

/// Check if there's snow below, and if there is add the current flake to
/// the snowbank and shuffle it to the top.
fn balance_bottom(flake: &mut Snowflake, snowbank: &mut Framebuffer) -> bool {
    let x = flake.x as usize;
    let y = flake.y as usize;
    let neighbour_left;
    let neighbour_right;
    let choice;

    if let Ok(value) = snowbank.get(x, y + 1) {
        // Check if we've hit the bottom
        if value == COLOUR_FLAKE {
            neighbour_left = snowbank.get(x + 1, y + 1).unwrap_or(COLOUR_FLAKE);
            neighbour_right = snowbank.get(x - 1, y + 1).unwrap_or(COLOUR_FLAKE);

            choice = random() * 2.0 < 1.0;
            
            // Move to the left or move to the right.
            if choice {
                if neighbour_left == COLOUR_BACKGROUND {
                    flake.x -= 1.0;
                    return false;
                } else if neighbour_right == COLOUR_BACKGROUND {
                    flake.x += 1.0;
                    return false;
                }
            } else {
                if neighbour_right == COLOUR_BACKGROUND {
                    flake.x += 1.0;
                    return false;
                } else if neighbour_left == COLOUR_BACKGROUND {
                    flake.x -= 1.0;
                    return false;
                }
            }

            let _ = snowbank.set(x, y, COLOUR_FLAKE);
            flake.randomize();
            flake.y = random() * RESPAWN_HEIGHT_JITTER * -1.0;

            return false
        }
    }

    true
}

struct LocalData {
    snowbank: Framebuffer,
    snowflakes: [Snowflake; FLAKE_COUNT],
    sine_table: SineTable,
}

impl LocalData {
    pub const fn new() -> Self {
        let snowbank = Framebuffer([COLOUR_BACKGROUND; WIDTH * HEIGHT]);
        let sine_table = [0.0f32; 628];
        let snowflakes = [Snowflake::new(); FLAKE_COUNT];

        Self {
            snowbank,
            snowflakes,
            sine_table,
        }
    }

    fn fill_snowbank(&mut self) {
        let length = self.snowbank.0.len();
        self.snowbank.0[length - (WIDTH * 2) .. length].fill(COLOUR_FLAKE);
    }

    fn randomize_snowflakes(&mut self) {
        for flake in self.snowflakes.iter_mut() {
            flake.randomize();
        }    
    }
        
    fn calculate_sine_table(&mut self) {
        for i in 0 .. self.sine_table.len() {
            self.sine_table[i] = sin((i / 100) as f32);
        }
    }

    fn clear_snowflakes(&mut self, image_buffer: &mut Framebuffer) {
        image_buffer.0.copy_from_slice(&self.snowbank.0);
    }

    fn draw_snowflakes(&mut self, image_buffer: &mut Framebuffer) {
        for flake in &mut self.snowflakes {
            if balance_bottom(flake, &mut self.snowbank) {
                flake.animate(&self.sine_table);
            }
    
            // If the flake can be drawn, do so. If somehow it can't, shuffle
            if image_buffer.set(flake.x as usize, flake.y as usize, COLOUR_FLAKE).is_err() {
                flake.randomize();
                flake.y = -1.0 * (random() * RESPAWN_HEIGHT_JITTER);
            }
        }
    
    }    
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

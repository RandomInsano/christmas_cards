#![no_std]

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern {
    fn js_random() -> f32;
    fn js_sin(x: f32) -> f32;
}

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

#[no_mangle]
/// What will be present in the browser
static mut BUFFER: Framebuffer = Framebuffer([0; WIDTH * HEIGHT]);
/// Buffer that stores the fallen snow and blanks the BUFFER each frame
static mut SNOWBANK: Framebuffer = Framebuffer([0; WIDTH * HEIGHT]);
/// Currently moving snowflakes
static mut SNOWFLAKES: [Snowflake; FLAKE_COUNT] = [Snowflake::new(); FLAKE_COUNT];
/// Lookup table to save us from calculating Sine for every flake every frame
static mut SIN_LOOKUP: SineTable = [0.0f32; 628];

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
}

#[no_mangle]
pub unsafe extern fn init() {
    init_safe(&mut SNOWFLAKES, &mut SNOWBANK, &mut SIN_LOOKUP);
}

#[no_mangle]
pub unsafe extern fn mouse_move(x: i32, y: i32) {
    mouse_move_safe(x, y, &mut SNOWFLAKES);
}

#[no_mangle]
pub unsafe extern fn go() {
    render_frame_safe(&mut BUFFER, &mut SNOWBANK, &mut SNOWFLAKES, &SIN_LOOKUP);
}

fn random() -> f32 {
    unsafe { js_random() }
}

fn sin(x: f32) -> f32 {
    unsafe { js_sin(x) }
}

/// Update position of a single snowflake
fn move_flake(flake: &mut Snowflake, sine_lookup: &SineTable) {
    flake.y += TERMINAL_VELOCITY;
    flake.seed = (flake.seed + SPIN_SPEED) % 628;

    flake.x += sine_lookup[flake.seed] * flake.radius;
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

/// Initialization goods
fn init_safe(snowflakes: &mut [Snowflake], snowbank: &mut Framebuffer, sine_table: &mut SineTable) {
    snowbank.0.fill(COLOUR_BACKGROUND);

    // Create two lines of snow at the bottom
    let length = snowbank.0.len();
    snowbank.0[length - (WIDTH * 2) .. length].fill(COLOUR_FLAKE);

    // Populate random positions for flakes to fall. Some are above the top line
    for flake in snowflakes {
        flake.randomize();
    }

    // Populate our sine lookup table
    for i in 0 .. sine_table.len() {
        sine_table[i] = sin((i / 100) as f32);
    }
}

fn mouse_move_safe(_x: i32, _y: i32, _snowflakes: &mut [Snowflake]) {
    /*
    for flake in snowflakes {
        let yf = flake.y as i32;
        let y1 = yf - 5;
        let y2 = yf + 5; 
        if y > y1 && y < y2 {
            flake.y -= TERMINAL_VELOCITY;
        }
    }
    */
}

/// Render the current frame
fn render_frame_safe(buffer: &mut Framebuffer, snowbank: &mut Framebuffer, snowflakes: &mut [Snowflake], sine_lookup: &SineTable) {
    // Clear the screen
    buffer.0.copy_from_slice(&snowbank.0);

    for flake in snowflakes {
        if balance_bottom(flake, snowbank) {
            move_flake(flake, sine_lookup);
        }

        // If the flake can be drawn, do so. If somehow it can't, fling it to
        // the top        
        if buffer.set(flake.x as usize, flake.y as usize, COLOUR_FLAKE).is_err() {
            flake.randomize();
            flake.y = -1.0 * (random() * RESPAWN_HEIGHT_JITTER);
        }
    }
}

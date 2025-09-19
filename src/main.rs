use macroquad::prelude::*;

const WINDOW_W: f32 = 1024.0;
const WINDOW_H: f32 = 768.0;

// Physical constants
const CP_WATER: f32 = 4186.0; // J/(kg*K)
const CP_ICE: f32 = 2100.0;   // J/(kg*K)
const LATENT_FUSION: f32 = 334_000.0; // J/kg
const U_EFFECTIVE: f32 = 5.0; // overall heat transfer (tunable)

// Visual mapping
const PIXELS_PER_KG: f32 = 120.0; // visual scale from kg -> px height

#[derive(Clone, Copy)]
struct SystemState {
    mass_water: f32,
    mass_ice: f32,
    mass_air: f32,
    temp_water: f32, // Celsius
    temp_ice: f32,   // Celsius
}

impl SystemState {
    // fn total_mass(&self) -> f32 {
    //     self.mass_water + self.mass_ice + self.mass_air
    // }

    fn system_temperature_equivalent(&self) -> f32 {
        // sensible heat weighted temperature relative to 0 °C:
        let sensible_ice = self.mass_ice * CP_ICE * self.temp_ice;
        let sensible_water = self.mass_water * CP_WATER * self.temp_water;
        let c_eff = self.mass_ice * CP_ICE + self.mass_water * CP_WATER;
        if c_eff.abs() < 1e-9 {
            0.0
        } else {
            (sensible_ice + sensible_water) / c_eff
        }
    }
}

struct Simulation {
    state: SystemState,
    outside_temp: f32,
    time_seconds: f32,
    running: bool,
    time_scale: f32, // multiplier 1,2,5,10

    // initial GUI-editable values
    init_water: f32,
    init_ice: f32,
    init_air: f32,
    init_system_temp: f32,
    init_outside_temp: f32,
}

impl Simulation {
    fn new() -> Self {
        let init_water = 0.5;
        let init_ice = 0.1;
        let init_air = 0.02;
        let init_temp = 5.0;
        let out_temp = 25.0;
        Self {
            state: SystemState {
                mass_water: init_water,
                mass_ice: init_ice,
                mass_air: init_air,
                temp_water: init_temp,
                temp_ice: init_temp.min(0.0),
            },
            outside_temp: out_temp,
            time_seconds: 0.0,
            running: false,
            time_scale: 1.0,
            init_water,
            init_ice,
            init_air,
            init_system_temp: init_temp,
            init_outside_temp: out_temp,
        }
    }

    fn reset_from_init(&mut self) {
        self.state.mass_water = self.init_water;
        self.state.mass_ice = self.init_ice;
        self.state.mass_air = self.init_air;
        self.state.temp_water = self.init_system_temp;
        self.state.temp_ice = self.init_system_temp.min(0.0);
        self.outside_temp = self.init_outside_temp;
        self.time_seconds = 0.0;
        self.running = false;
        self.time_scale = 1.0;
    }

    fn step(&mut self, dt: f32) {
        if !self.running {
            return;
        }
        let dt = dt * self.time_scale;

        // Equivalent system temp (sensible)
        let sys_temp = self.state.system_temperature_equivalent();

        // Heat flow from outside -> system (positive => heating)
        let q_dot = U_EFFECTIVE * (self.outside_temp - sys_temp); // J/s
        let mut q = q_dot * dt; // Joules delivered during dt

        // HEATING (q > 0): raise ice temp to 0, melt, then heat water
        if q > 0.0 {
            // 1) warm ice to 0°C
            if self.state.mass_ice > 0.0 && self.state.temp_ice < 0.0 {
                let need = self.state.mass_ice * CP_ICE * (0.0 - self.state.temp_ice);
                if q >= need {
                    self.state.temp_ice = 0.0;
                    q -= need;
                } else {
                    self.state.temp_ice += q / (self.state.mass_ice * CP_ICE);
                    q = 0.0;
                }
            }

            // 2) melt ice at 0°C
            if q > 0.0 && self.state.mass_ice > 0.0 {
                let can_melt = q / LATENT_FUSION;
                let melt_mass = can_melt.min(self.state.mass_ice);
                self.state.mass_ice -= melt_mass;
                self.state.mass_water += melt_mass;
                q -= melt_mass * LATENT_FUSION;
                // melted water enters at 0°C; we will mix below
            }

            // 3) raise water temperature (mixed water)
            if q > 0.0 && self.state.mass_water > 0.0 {
                let delta_t = q / (self.state.mass_water * CP_WATER);
                self.state.temp_water += delta_t;
                // q = 0.0;
            }
        } else if q < 0.0 {
            // COOLING: remove energy from water down to 0°C, freeze, then cool ice
            let mut q_abs = -q;

            // 1) cool water to 0°C
            if self.state.mass_water > 0.0 && self.state.temp_water > 0.0 {
                let need = self.state.mass_water * CP_WATER * (self.state.temp_water - 0.0);
                let take = need.min(q_abs);
                self.state.temp_water -= take / (self.state.mass_water * CP_WATER);
                q_abs -= take;
            }

            // 2) freeze some water at 0°C (latent)
            if q_abs > 0.0 && self.state.mass_water > 0.0 && (self.state.temp_water - 0.0).abs() < 1e-3 {
                let freeze_mass = (q_abs / LATENT_FUSION).min(self.state.mass_water);
                self.state.mass_water -= freeze_mass;
                self.state.mass_ice += freeze_mass;
                q_abs -= freeze_mass * LATENT_FUSION;
            }

            // 3) lower ice temperature
            if q_abs > 0.0 && self.state.mass_ice > 0.0 {
                let delta_t = q_abs / (self.state.mass_ice * CP_ICE);
                self.state.temp_ice -= delta_t;
                // q_abs = 0.0;
            }

            // negative q handled, set q = 0 implicitly
        }

        // Ensure temp bounds and mass sanity
        if self.state.mass_ice > 0.0 {
            self.state.temp_ice = self.state.temp_ice.min(0.0);
        } else {
            self.state.temp_ice = 0.0;
        }
        if self.state.mass_water > 0.0 {
            self.state.temp_water = self.state.temp_water.max(0.0);
        } else {
            // if no water, keep temp at 0 (degenerate)
            self.state.temp_water = 0.0;
        }

        self.time_seconds += dt;
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Bottle Thermal Simulation".to_string(),
        window_width: WINDOW_W as i32,
        window_height: WINDOW_H as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    let mut sim = Simulation::new();
    let mut selected_field: usize = 0;
    let fields = [
        "Init water (kg)",
        "Init ice (kg)",
        "Init air (kg)",
        "Init system temp (C)",
        "Outside temp (C)",
    ];

    loop {
        clear_background(Color::from_rgba(18, 20, 28, 255));

        let dt = get_frame_time();
        sim.step(dt);

        // Layout sizes
        let left_card_x = 12.0;
        let left_card_y = 12.0;
        let left_card_w = 300.0;
        let left_card_h = 160.0;

        let right_card_w = 300.0;
        let right_card_x = WINDOW_W - right_card_w - 12.0;
        let right_card_y = 12.0;

        // Bottle position - centered between the UI cards
        let bottle_center_x = WINDOW_W / 2.0;
        let bottle_w = 220.0;
        let bottle_h = 420.0;
        let bottle_x = bottle_center_x - bottle_w / 2.0;
        let bottle_y = WINDOW_H / 2.0 - bottle_h / 2.0;

        // Draw bottle body
        let top_center = vec2(bottle_center_x, bottle_y);
        draw_rectangle(top_center.x - 45., top_center.y - 7., bottle_w * 0.38, 16., GRAY);

        draw_rectangle(bottle_x, bottle_y + 10.0, bottle_w, bottle_h - 10.0, Color::from_rgba(20, 30, 50, 80));
        draw_rectangle_lines(bottle_x, bottle_y + 10.0, bottle_w, bottle_h - 10.0, 3.0, GRAY);

        // compute liquid height
        let liquid_mass = sim.state.mass_water + sim.state.mass_ice;
        let liquid_height_px = (liquid_mass * PIXELS_PER_KG).min(bottle_h - 12.0);
        let water_fraction = if liquid_mass > 0.0 {
            sim.state.mass_water / liquid_mass
        } else {
            0.0
        };
        let water_height_px = liquid_height_px * water_fraction;
        let ice_height_px = liquid_height_px - water_height_px;

        let water_top = bottle_y + bottle_h - water_height_px - 6.0;
        if sim.state.mass_water > 0.0 {
            // water rectangle
            draw_rectangle(bottle_x + 4.0, water_top, bottle_w - 8.0, water_height_px.max(1.0), Color::from_rgba(30, 90, 200, 200));
            // water surface ellipse
            draw_rectangle(bottle_x + 4.0, water_top, bottle_w - 8.0, water_height_px.max(1.0), Color::from_rgba(30, 90, 200, 200));
            draw_line(bottle_x + 4.0, water_top, bottle_x + bottle_w - 4.0, water_top, 2.0, Color::from_rgba(50, 140, 220, 200));
        }

        // ice blocks drawn stacked above water
        let mut ice_y = water_top - ice_height_px;
        let mut remaining = ice_height_px;
        while remaining > 0.0 {
            let block_h = remaining.min(36.0);
            draw_rectangle(bottle_x + 8.0, ice_y, bottle_w - 16.0, block_h.max(1.0), Color::from_rgba(230, 245, 255, 230));
            draw_rectangle_lines(bottle_x + 8.0, ice_y, bottle_w - 16.0, block_h.max(1.0), 1.0, Color::from_rgba(180, 200, 220, 200));
            ice_y += block_h;
            remaining -= block_h;
        }

        // Top-left status card
        draw_rectangle(left_card_x, left_card_y, left_card_w, left_card_h, Color::from_rgba(8, 8, 12, 220));
        draw_rectangle_lines(left_card_x, left_card_y, left_card_w, left_card_h, 2.0, LIGHTGRAY);
        draw_text(&format!("Time: {:.1} s", sim.time_seconds), left_card_x + 10.0, left_card_y + 28.0, 20.0, WHITE);
        draw_text(&format!("Water: {:.4} kg", sim.state.mass_water), left_card_x + 10.0, left_card_y + 56.0, 18.0, WHITE);
        draw_text(&format!("Ice:   {:.4} kg", sim.state.mass_ice), left_card_x + 10.0, left_card_y + 82.0, 18.0, WHITE);
        draw_text(&format!("T_water: {:.2} °C", sim.state.temp_water), left_card_x + 10.0, left_card_y + 108.0, 18.0, WHITE);
        draw_text(&format!("T_ice:   {:.2} °C", sim.state.temp_ice), left_card_x + 10.0, left_card_y + 134.0, 18.0, WHITE);

        // Top-right controls card
        let ctrl_h = 250.0;
        draw_rectangle(right_card_x, right_card_y, right_card_w, ctrl_h, Color::from_rgba(8, 8, 12, 220));
        draw_rectangle_lines(right_card_x, right_card_y, right_card_w, ctrl_h, 2.0, LIGHTGRAY);
        draw_text(
            "Ctrls: Tab: field, +/-: change, Enter: Start/Pause",
            right_card_x + 8.0,
            right_card_y + 22.0,
            13.0,
            LIGHTGRAY,
        );

        // editable fields listing (highlight selected)
        let vals = [
            sim.init_water,
            sim.init_ice,
            sim.init_air,
            sim.init_system_temp,
            sim.init_outside_temp,
        ];
        let mut fy = right_card_y + 46.0;
        for i in 0..5 {
            let is_sel = i == selected_field;
            let bg = if is_sel { Color::from_rgba(36, 36, 50, 220) } else { Color::from_rgba(0, 0, 0, 0) };
            draw_rectangle(right_card_x + 8.0, fy - 18.0, right_card_w - 16.0, 28.0, bg);
            draw_text(&format!("{:20}: {:.3}", fields[i], vals[i]), right_card_x + 14.0, fy, 16.0, WHITE);
            fy += 36.0;
        }

        // Buttons (Start, Reset, Speed)
        let btn_y = right_card_y + ctrl_h - 40.0;
        let btn_w = 87.0;
        let btn_h = 34.0;
        let start_label = if sim.running { "Pause" } else { "Start" };
        draw_rectangle(right_card_x + 3.0, btn_y, btn_w, btn_h, Color::from_rgba(60, 120, 60, 220));
        draw_text(start_label, right_card_x + 12.0 + 14.0, btn_y + 24.0, 18.0, WHITE);

        draw_rectangle(right_card_x + 12.0 + btn_w + 8.0, btn_y, btn_w, btn_h, Color::from_rgba(150, 60, 60, 220));
        draw_text("Reset", right_card_x + 12.0 + btn_w + 12.0 + 22.0, btn_y + 24.0, 18.0, WHITE);

        draw_rectangle(right_card_x + 12.0 + 2.0 * (btn_w + 12.0), btn_y, btn_w, btn_h, Color::from_rgba(60, 60, 120, 220));
        draw_text(&format!("Speed x{}", sim.time_scale as i32), right_card_x + 12.0 + 2.0 * (btn_w + 12.0) + 10.0, btn_y + 24.0, 16.0, WHITE);

        // Mouse clicks for buttons
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            // Start/Pause
            if mx >= right_card_x + 12.0 && mx <= right_card_x + 12.0 + btn_w && my >= btn_y && my <= btn_y + btn_h {
                // apply inits if paused
                if !sim.running {
                    sim.state.mass_water = sim.init_water;
                    sim.state.mass_ice = sim.init_ice;
                    sim.state.mass_air = sim.init_air;
                    sim.state.temp_water = sim.init_system_temp;
                    sim.state.temp_ice = sim.init_system_temp.min(0.0);
                    sim.outside_temp = sim.init_outside_temp;
                }
                sim.running = !sim.running;
            }
            // Reset
            if mx >= right_card_x + 12.0 + btn_w + 12.0 && mx <= right_card_x + 12.0 + 2.0 * btn_w + 12.0 && my >= btn_y && my <= btn_y + btn_h {
                sim.reset_from_init();
            }
            // Speed toggle
            if mx >= right_card_x + 12.0 + 2.0 * (btn_w + 12.0) && mx <= right_card_x + 12.0 + 3.0 * btn_w + 24.0 && my >= btn_y && my <= btn_y + btn_h {
                sim.time_scale = match sim.time_scale as i32 {
                    1 => 2.0,
                    2 => 5.0,
                    5 => 10.0,
                    _ => 1.0,
                };
            }
        }

        // Keyboard input
        if is_key_pressed(KeyCode::Tab) {
            selected_field = (selected_field + 1) % 5;
        }
        // Adjust selected field by small increments
        let mut delta = 0.0;
        if is_key_down(KeyCode::KpAdd) || is_key_down(KeyCode::Up) {
            delta = 0.01;
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                delta = 0.1;
            }
        }
        if is_key_down(KeyCode::KpSubtract) || is_key_down(KeyCode::Down) {
            delta = -0.01;
            if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                delta = -0.1;
            }
        }
        if delta != 0.0 {
            match selected_field {
                0 => sim.init_water = (sim.init_water + delta).max(0.0),
                1 => sim.init_ice = (sim.init_ice + delta).max(0.0),
                2 => sim.init_air = (sim.init_air + delta).max(0.0),
                3 => sim.init_system_temp = sim.init_system_temp + delta * 5.0,
                4 => sim.init_outside_temp = sim.init_outside_temp + delta * 5.0,
                _ => {}
            }
        }

        if is_key_pressed(KeyCode::Enter) {
            if !sim.running {
                sim.state.mass_water = sim.init_water;
                sim.state.mass_ice = sim.init_ice;
                sim.state.mass_air = sim.init_air;
                sim.state.temp_water = sim.init_system_temp;
                sim.state.temp_ice = sim.init_system_temp.min(0.0);
                sim.outside_temp = sim.init_outside_temp;
            }
            sim.running = !sim.running;
        }
        if is_key_pressed(KeyCode::R) {
            sim.reset_from_init();
        }
        if is_key_pressed(KeyCode::S) {
            sim.time_scale = match sim.time_scale as i32 {
                1 => 2.0,
                2 => 5.0,
                5 => 10.0,
                _ => 1.0,
            };
        }

        // Legend & FPS
        draw_text("Model: simplified lumped heat + latent melt.", 12.0, WINDOW_H - 44.0, 16.0, LIGHTGRAY);
        draw_text(&format!("FPS: {}", get_fps()), WINDOW_W - 96.0, WINDOW_H - 24.0, 16.0, LIGHTGRAY);

        next_frame().await;
    }
}

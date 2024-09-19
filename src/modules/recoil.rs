use std::fs::File;
use std::io::BufWriter;
use std::thread;
use std::time::{Duration, Instant};
use hid_api_rs::gadgets::mouse;
use hid_api_rs::gadgets::mouse::MouseRaw;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct RecoilSettings {
    #[structopt(short, long, default_value = "0")]
    /// Vertical recoil
    pub vertical: f32,

    #[structopt(short, long, default_value = "0")]
    /// Initial recoil
    pub initial: f32,

    #[structopt(short, long, default_value = "0")]
    /// Gun's Rounds-per-minute (RPM)
    pub rpm: i16,

    #[structopt(short, long, default_value = "30")]
    /// Magazine size
    pub mag_size: i16,

    #[structopt(short, long, default_value = "120")]
    /// Game's internal FOV
    pub fov: i16,

    #[structopt(short = "sn", long, default_value = "100")]
    /// Game's Sensitivity
    pub sensitivity: i16,

    #[structopt(short = "sm", long)]
    /// Program's smoothness
    pub smoothness: Option<i16>,

    #[structopt(short = "sc", long, default_value = "1")]
    /// Scope magnification level (1.5x, 2x, 2.5x)
    pub scope: f32,

    #[structopt(short, long)]
    /// Global overflow
    pub global_overflow: bool,

    #[structopt(short, long)]
    /// Local overflow
    pub local_overflow: bool
}

impl Default for RecoilSettings {
    fn default() -> Self {
        RecoilSettings {
            vertical: 0.0,
            initial: 0.0,
            rpm: 500,
            mag_size: 30,
            fov: 120,
            sensitivity: 100,
            smoothness: None,
            scope: 1.0,
            global_overflow: false,
            local_overflow: false
        }
    }
}

pub fn start_recoil_handler(recoil_settings: RecoilSettings, gadget_file: File) {
    if recoil_settings.vertical == 0.0 {
        return;
    }

    let mut global_overflow_y: Decimal = dec!(0.0);
    let mut global_time_overflow: Decimal = dec!(0.0);
    let mut current_bullet: i16 = 0;


    let multiplier: Decimal = Decimal::from(recoil_settings.fov) * (dec!(12.0) / dec!(60.0)) / (Decimal::from(recoil_settings.sensitivity) / dec!(100.0));

    // ROUNDS PER MILLISECOND
    let delay = dec!(60000.0) / Decimal::from(recoil_settings.rpm);

    if let Some(smoothness) = recoil_settings.smoothness {
        println!("Using user-defined smoothness value ({})", smoothness)
    }

    let best_smoothness = match recoil_settings.smoothness {
        Some(smoothness) => Decimal::from(smoothness),
        None => (Decimal::from_f32_retain(recoil_settings.vertical).unwrap() * Decimal::from_f32_retain(recoil_settings.scope).unwrap() * multiplier).sqrt().unwrap().round()
    };

    let smoothed_delay_ns = (delay / best_smoothness) * dec!(1000000);

    let mut recoil_stopwatch_reset: Instant = Instant::now();
    let mut recoil_stopwatch_started: bool = false;

    let mut gadget_writer = BufWriter::with_capacity(16, gadget_file);

    let mouses = hid_api_rs::get_mouses();

    loop {
        if mouses.is_empty() {
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        break;
    }

    // Assume first mouse is best mouse.
    let mouse = mouses.get_mut(0).unwrap();

    let mut overflow_y;
    let mut time_overflow ;

    loop {
        let perf = Instant::now();
        let mouse_state = *mouse.get_state();

        if mouse_state.left_button && mouse_state.right_button {
            recoil_stopwatch_reset = Instant::now();

            if current_bullet < recoil_settings.mag_size {
                let mut local_y = Decimal::from_f32_retain(recoil_settings.vertical).unwrap() * Decimal::from_f32_retain(recoil_settings.scope).unwrap();

                if current_bullet == 0 {
                    local_y = local_y.powd(Decimal::from_f32_retain(recoil_settings.initial).unwrap());
                }

                local_y *= multiplier;

                overflow_y = dec!(0.0);

                if recoil_settings.local_overflow && recoil_settings.global_overflow {
                    if global_overflow_y.trunc() != dec!(0.0) {
                        let truncated_global_overflow_y = global_overflow_y.round();
                         global_overflow_y -= truncated_global_overflow_y;

                        local_y += truncated_global_overflow_y;
                    }
                }

                println!("Bullet {current_bullet} \\ {} | (Y: {local_y}, BSM: {best_smoothness}, RPM {}) | (GOverflow: {global_overflow_y}, Computation: {})", recoil_settings.mag_size, recoil_settings.rpm, perf.elapsed().as_nanos());

                time_overflow = global_time_overflow;
                global_time_overflow = dec!(0.0);

                for _ in 0..best_smoothness.to_i16().unwrap() {
                    let smoothed_y = local_y  / best_smoothness;
                    let mut smoothed_int_y = smoothed_y.round();

                    if recoil_settings.local_overflow {
                        overflow_y += smoothed_y - smoothed_int_y;

                        if overflow_y.trunc() != dec!(0.0) {
                            let truncated_overflow_y = overflow_y.round();
                            overflow_y -= truncated_overflow_y;

                            smoothed_int_y += truncated_overflow_y;
                        }
                    }

                    _ = mouse::push_mouse_event(MouseRaw {
                        relative_x: 0,
                        relative_y: smoothed_int_y.to_i16().unwrap(),
                        left_button: mouse_state.left_button,
                        middle_button: mouse_state.middle_button,
                        right_button: mouse_state.right_button,
                        four_button: mouse_state.four_button,
                        five_button: mouse_state.five_button,
                        relative_wheel: 0
                    }, None, &mut gadget_writer);

                    let bullet_timing = Instant::now();

                    loop {
                        let elapsed = bullet_timing.elapsed().as_nanos();

                        if elapsed >= (smoothed_delay_ns - time_overflow).to_u128().unwrap() {
                            time_overflow = Decimal::from(elapsed) - smoothed_delay_ns;
                            break;
                        }
                    }
                }

                global_time_overflow += time_overflow;
                global_overflow_y += overflow_y;
                current_bullet += 1;
            }

            continue;
        }

        if !recoil_stopwatch_started {
            recoil_stopwatch_reset = Instant::now();

            recoil_stopwatch_started = true;
        }

        if recoil_stopwatch_reset.elapsed().as_micros() >= 250 {
            current_bullet = 0;
            global_overflow_y = dec!(0.0);
            global_time_overflow = dec!(0.0);

            recoil_stopwatch_started = false;
        }
    }
}
#![no_std]
#![feature(type_alias_impl_trait, const_async_blocks)]
#![warn(
    clippy::complexity,
    clippy::correctness,
    clippy::perf,
    clippy::style,
    clippy::undocumented_unsafe_blocks,
    rust_2018_idioms
)]

use asr::{
    emulator::gcn::{self, Emulator},
    future::{next_tick, retry},
    watcher::Watcher, timer::{self, TimerState}, time::Duration,
};
use asr::time_util;

asr::panic_handler!();
asr::async_main!(nightly);

async fn main() {
    let settings = Settings::register();

    loop {
        // Hook to the target process
        let mut emulator = retry(|| gcn::Emulator::attach()).await;
        let mut watchers = Watchers::default();
        let offsets = Offsets::new();
        
        loop {
            if !emulator.is_open() {
                break;
            }

            if emulator.update() {
                // Splitting logic. Adapted from OG LiveSplit:
                // Order of execution
                // 1. update() will always be run first. There are no conditions on the execution of this action.
                // 2. If the timer is currently either running or paused, then the isLoading, gameTime, and reset actions will be run.
                // 3. If reset does not return true, then the split action will be run.
                // 4. If the timer is currently not running (and not paused), then the start action will be run.
                update_loop(&emulator, &offsets, &mut watchers);

                let timer_state = timer::state();
                if timer_state == TimerState::Running || timer_state == TimerState::Paused {
                    if let Some(is_loading) = is_loading(&watchers, &settings) {
                        if is_loading {
                            timer::pause_game_time()
                        } else {
                            timer::resume_game_time()
                        }
                    }

                    if let Some(game_time) = game_time(&watchers, &settings) {
                        timer::set_game_time(game_time)
                    }

                    if reset(&watchers, &settings) {
                        timer::reset()
                    } else if split(&watchers, &settings) {
                        timer::split()
                    }
                }

                if timer::state() == TimerState::NotRunning && start(&watchers, &settings) {
                    timer::start();
                    timer::pause_game_time();

                    if let Some(is_loading) = is_loading(&watchers, &settings) {
                        if is_loading {
                            timer::pause_game_time()
                        } else {
                            timer::resume_game_time()
                        }
                    }
                }
            }
            next_tick().await;
        }
    }
}

#[derive(asr::user_settings::Settings)]
struct Settings {
    #[default = true]
    /// START --> Enable auto start
    start: bool,
}

#[derive(Default)]
struct Watchers {
    igt: Watcher<f32>,
}

struct Offsets {
    score: u32,
}

impl Offsets {
    fn new() -> Self {
        Self {
            score: 0x93DC,
        }
    }
}


fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {
    let score = game.read::<u32>(offsets.score).unwrap_or_default();

    watchers.igt.update_infallible(1.0);
}

fn start(watchers: &Watchers, settings: &Settings) -> bool {
    if !settings.start {
        return false;
    }
    false
}

fn split(watchers: &Watchers, settings: &Settings) -> bool {
    false
}

fn reset(watchers: &Watchers, settings: &Settings) -> bool {
    false
}

fn is_loading(watchers: &Watchers, settings: &Settings) -> Option<bool> {
    Some(true)
}

fn game_time(watchers: &Watchers, settings: &Settings) -> Option<Duration> {
    let current_frames = 3600;

    let seconds = 3.0;

    let igt = Duration::seconds_f32(seconds);


    let igt = time_util::frame_count::<60>(current_frames);

    Some(igt)
}
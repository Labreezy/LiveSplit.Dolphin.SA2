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

use core::cmp::max;

use asr::{
    print_limited,
    emulator::gcn::{self, Emulator},
    future::{next_tick, retry},
    watcher::Watcher, timer::{self, TimerState}, time::Duration, time_util::frame_count, print_message,
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
        let mut countFrames : bool = false;
        let offsets = Offsets::new();
        let mut igt_info = IGTInfo::default();

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
                if timer_state == TimerState::Running {
                    

                    if let Some(game_time) = game_time(&watchers, &settings, &mut igt_info) {
                        print_message("gaming time");
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
struct IGTInfo {
    igt_frames: u64,
    igt_duration: Duration,
}
#[derive(Default)]
struct Watchers {
    level_timer_mins: Watcher<u8>,
    level_timer_secs: Watcher<u8>,
    level_timer_centis: Watcher<u8>,
    frame_counter: Watcher<u32>,
    gamestate_flags: Watcher<u8>,
}

struct Offsets {
    level_timer_mins: u32,
    level_timer_secs: u32,
    level_timer_centis: u32,
    frame_counter: u32,
    gamestate_flags: u32,
}

impl Offsets {
    fn new() -> Self {
        Self {
            level_timer_mins: 0x1B3D2F,
            level_timer_secs: 0x1B3D6F,
            level_timer_centis: 0x1B3DAF,
            frame_counter: 0x1CC1E0,
            gamestate_flags: 0x3AD81B,
        }
    }
}


fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {
    let timer_minutes = game.read::<u8>(offsets.level_timer_mins).unwrap_or_default();
    let timer_seconds = game.read::<u8>(offsets.level_timer_secs).unwrap_or_default();
    let timer_centis = game.read::<u8>(offsets.level_timer_centis).unwrap_or_default();
    let fc = game.read::<u32>(offsets.frame_counter).unwrap_or_default();
    let stateflags = game.read::<u8>(offsets.gamestate_flags).unwrap_or_default();
    
    watchers.level_timer_mins.update_infallible(timer_minutes);
    watchers.level_timer_secs.update_infallible(timer_seconds);
    watchers.level_timer_centis.update_infallible(timer_centis);
    watchers.frame_counter.update_infallible(fc);
    watchers.gamestate_flags.update_infallible(stateflags);
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

fn game_time(watchers: &Watchers, settings: &Settings, info: &mut IGTInfo) -> Option<Duration> {
    let mut countFrames = false;
    let mut igt = frame_count::<60>(info.igt_frames);

    let Some(minutes) = watchers.level_timer_mins.pair else {return Some(igt)};
    let Some(seconds) = watchers.level_timer_secs.pair else {return Some(igt)};
    let Some(centis) = watchers.level_timer_centis.pair else {return Some(igt)};
    let Some(fcount) = watchers.frame_counter.pair else {return Some(igt)};
    let Some(flags) = watchers.gamestate_flags.pair else {return Some(igt)};
    if seconds.changed()|| minutes.changed() || centis.changed(){
        countFrames = true;
    }  else if flags.current == 17 || flags.current == 16 { //paused but ingame, ingame
        countFrames = true;
    }
    let framesToAdd : u32;
    if countFrames {
        framesToAdd = max(fcount.current - fcount.old, 0);
        info.igt_frames = info.igt_frames + (framesToAdd as u64);
    }
    igt = frame_count::<60>(info.igt_frames);
    info.igt_duration = igt;
    Some(igt)
}
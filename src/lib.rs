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
    watcher::Watcher, timer::{self, TimerState}, time::Duration, time_util::frame_count, print_message, Address32, Address,
};
use asr::time_util;

use bitflags::bitflags;

asr::panic_handler!();
asr::async_main!(nightly);


const POWERUP_DEAD : u16 = 0x4000;
const VALID_CENTIS : [f64; 6] = [0.,0.02,0.04,0.05,0.07,0.09];

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
                        
                        timer::set_game_time(game_time)
                    }

                    if reset(&watchers, &settings) {
                        timer::reset();


                        
                    } else if split(&watchers, &settings) {
                        timer::split()
                    }
                }

                if timer::state() == TimerState::NotRunning {

                    igt_info = IGTInfo::default();
                    
                    if start(&watchers, &settings) {
                    timer::start();
                    timer::pause_game_time();
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
    /// PLACEHOLDER SETTING, DOES NOTHING   
    start: bool,
}
#[derive(Default)]
struct IGTInfo {
    igt_frames: u64,
    igt_duration: Duration,
    last_timer: [u8;3],
    paused_frames: u64,
}
#[derive(Default)]
struct Watchers {
    level_timer: Watcher<u32>,
    level_timer2: Watcher<u32>,
    frame_counter: Watcher<u32>,
    gamestate_flags: Watcher<u8>,
    powerups_bitfield: Watcher<u16>,
    can_control_p1: Watcher<u8>,
}

struct Offsets {
    level_timer: u32,
    timer2: u32,
    frame_counter: u32,
    gamestate_flags: u32,
    p1_region: u32,
    p2_region: u32,
    powerup_offset: u8,
    can_control_p1: u32,

}

impl Offsets {
    fn new() -> Self {
        Self {
            level_timer: 0x1CC182,
            timer2: 0x1E530F,
            frame_counter: 0x1CC1E0,
            gamestate_flags: 0x3AD81B,
            p1_region: 0x1E7728,
            p2_region: 0x1E772C, 
            powerup_offset: 0x10,
            can_control_p1: 0x1CC1A7,
            //level_end: 0x1CC1AA,
        }
    }
}


fn update_loop(game: &Emulator, offsets: &Offsets, watchers: &mut Watchers) {
    let level_timer = game.read::<u32>(offsets.level_timer).unwrap_or_default();
    let fc = game.read::<u32>(offsets.frame_counter).unwrap_or_default();
    let stateflags = game.read::<u8>(offsets.gamestate_flags).unwrap_or_default();
    let controlp1 = game.read::<u8>(offsets.can_control_p1).unwrap_or_default();
    let mut igt2 = game.read::<u32>(offsets.timer2).unwrap_or_default();
    igt2 = igt2 & 0xFFFFFF;     
    watchers.level_timer.update_infallible(level_timer);
    watchers.frame_counter.update_infallible(fc);
    watchers.gamestate_flags.update_infallible(stateflags);
    watchers.can_control_p1.update_infallible(controlp1);
    watchers.level_timer2.update_infallible(igt2);
    let p1_region_base = game.read::<u32>(offsets.p1_region).unwrap_or_default();
    
    if p1_region_base > 0x8000000 {
        let powerups = game.read::<u16>(p1_region_base + offsets.powerup_offset as u32).unwrap_or_default();
        watchers.powerups_bitfield.update_infallible(powerups);
    }

}
    

fn start(watchers: &Watchers, settings: &Settings) -> bool {
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

fn to_sa2_ms(frames : u64) -> Duration {
    let subsecond_frames = frames % 60;
    let seconds = (frames - subsecond_frames)/60;
    let deciseconds = subsecond_frames/6;
    let centiseconds = VALID_CENTIS[(subsecond_frames as usize % 6)]; //i fuckin hate sa2 rounding
    let total_time = Duration::seconds_f64(seconds as f64 + deciseconds as f64/10.0 + centiseconds);
    print_limited::<32>(&format_args!("igt: {:.2}", total_time));
    return total_time
}

fn game_time(watchers: &Watchers, settings: &Settings, info: &mut IGTInfo) -> Option<Duration> {
    let mut countFrames = false;
    

    let Some(leveltime) = watchers.level_timer.pair else {return None};
    let Some(leveltime2) = watchers.level_timer2.pair else {return None};
    let Some(fcount) = watchers.frame_counter.pair else {return None};
    let Some(flags) = watchers.gamestate_flags.pair else {return None};
    let Some(powerups) = watchers.powerups_bitfield.pair else {return None};
    let Some(controlp1) = watchers.can_control_p1.pair else {return None};
    let framesToAdd : u32;
    if flags.current == 17 {
            if leveltime.current > 2  { //dont fucking @ me
                framesToAdd = max(fcount.current - fcount.old, 0);
                info.paused_frames = info.paused_frames + (framesToAdd as u64);
            }
    } else if flags.current == 16 && leveltime2.changed() {
        print_limited::<32>(&format_args!("{:x}",leveltime2.current));
        let mins_frames = (leveltime2.current >> 0x10) & 0xFF;
        let secs_frames = (leveltime2.current >> 8) & 0xFF;
        let centis_frames = (leveltime2.current) & 0xFF; //represented in frames, not centiseconds like PC
        let curr_igt = mins_frames * 3600 + secs_frames * 60 + centis_frames;
        let old_mins = (leveltime2.old >> 0x10) & 0xFF;
        let old_secs = (leveltime2.old >> 8) & 0xFF;
        let old_frames = (leveltime2.old) & 0xFF;
        let old_igt = old_mins * 3600 + old_secs * 60 + old_frames;
        let igt_diff : i32 = (curr_igt - old_igt) as i32;
        framesToAdd = max(igt_diff,0) as u32; //only positive igt
        info.igt_frames += framesToAdd as u64;
    }
    let total_igt = to_sa2_ms(info.igt_frames) + frame_count::<60>(info.paused_frames);
    Some(total_igt)
    
}
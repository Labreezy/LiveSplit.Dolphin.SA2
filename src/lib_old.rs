#![no_std]
#![feature(const_async_blocks)]
#![feature(type_alias_impl_trait)]



asr::async_main!(nightly);
asr::panic_handler!();



use asr::{Address,
    future::{next_tick, sleep}, 
    user_settings::Settings,
    signature::Signature, Process};
use bytemuck::CheckedBitPattern;

use core::{time::Duration};
use core::cmp::min;
mod state;

#[derive(Settings)]
struct Settings {
    //Auto-start upon starting story
    #[default = true]
    start_upon_story_start: bool,
   
}

const GC_SIG : Signature<10> = Signature::new(
    "47 53 4E 45 38 50 00 00 01 00"
);

macro_rules! unwrap_or_continue {
    ( $e:expr ) => {
        match $e {
            Some(x) => x,
            None => continue,
        }
    };
}

async fn main() {
    // TODO: Set up some general state and settings.
    let _settings = Settings::register();

    asr::print_message("Hi there!");

    loop {
        let process = Process::wait_attach("Dolphin.exe").await;
        process
            .until_closes(async {
                let mut dolphin_base: Option<Address> = None;
                while dolphin_base.is_none() {
                    asr::print_message("Didn't find it :(");
                    sleep(Duration::from_secs(2)).await;
                    dolphin_base = get_gc_base(&process);
                }
                loop {
                    // TODO: Do something on every tick.
                        
                    next_tick().await;
                    break;
                }
            })
            .await;
    }
}

fn get_gc_base(proc: &Process) -> Option<Address> {
    for range in proc.memory_ranges() {
        if let (Ok(address), Ok(size)) = (range.address(), range.size()) {
            asr::print_limited::<32>(&format_args!("0x{:x}", address.value())); 
            let base_addr = GC_SIG.scan_process_range(proc, (address, size)); //guaranteed to be within the first few bytes of a region
            if base_addr.is_some() {
                return base_addr;
            }
        }
    }

    None
}

fn read_dolphin<T: CheckedBitPattern+Default>(proc: &Process, base: Address, mut offset: u32) -> T{
    if offset > 0x4000000 {
        offset = offset & 0xFFFFFF;
    }
    let target_address : Address = base.add(offset as u64);
    proc.read::<T>(target_address).ok().unwrap_or_default()
}
#![no_std]
#![feature(const_async_blocks)]
#![feature(type_alias_impl_trait)]



asr::async_main!(nightly);
asr::panic_handler!();



use asr::{Address,
    future::{next_tick, sleep}, 
    user_settings::Settings
    signature::Signature, Process};

use core::{time::Duration};

mod state;

#[derive(Settings)]
struct Settings {
    //Auto-start upon starting story
    #[default = true]
    start_upon_story_start: bool,
   
}

const GC_SIG : Signature<8> = Signature::new(
    "47 53 4E 45 38 50 00 00"
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
                    let 
                    next_tick().await;
                    break;
                }
            })
            .await;
    }
}

fn get_gc_base(proc: &Process) -> Option<Address> {
    for range in proc.memory_ranges() {
        if let (Ok(address), Ok(_size)) = (range.address(), range.size()) {
            let base_addr = GC_SIG.scan_process_range(proc, (address, 0x100)); //guaranteed to be within the first few bytes of a region
            if base_addr.is_some() {
                return base_addr;
            }
        }
    }

    None
}

fn read_dolphin<T>(proc: &Process, base: Address, offset: u32){
    
}
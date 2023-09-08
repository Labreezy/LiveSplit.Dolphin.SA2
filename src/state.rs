#![no_std]

use asr::{watcher::{Watcher, Pair}};

pub struct GameState{
    pub igtMinutes: Watcher<u8>,
    pub igtSeconds: Watcher<u8>,
    pub igtCentis: Watcher<u8>,
}



pub struct GameStatePair {
    pub igtMinutes: Pair<u8>,
    pub igtSeconds: Pair<u8>,
    pub igtCentis: Pair<u8>,
}
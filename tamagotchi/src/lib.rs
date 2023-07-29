#![no_std]
use gstd::{debug, exec, msg, prelude::*};
use tamagotchi_io::TamagotchiState;

static mut STATE: Option<TamagotchiState> = None;

#[no_mangle]
extern "C" fn init() {
    let tamagotchi = TamagotchiState {
        name: msg::load().expect("no name given"),
        date_of_birth: exec::block_timestamp(),
    };
    debug!("Tamagotchi info: {:?}", tamagotchi);
    unsafe {
        STATE = Some(tamagotchi);
    }
}

#[no_mangle]
extern "C" fn handle() {}

#[no_mangle]
extern "C" fn state() {
    let tamagotchi = unsafe { STATE.as_ref().expect("tamagotchi not initialized") };
    msg::reply(tamagotchi, 0).expect("reply failed");
}

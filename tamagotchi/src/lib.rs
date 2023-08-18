#![no_std]
use gstd::{debug, exec, msg, prelude::*};
use tamagotchi_io::{TamagotchiState, TmAction, TmEvent};

static mut STATE: Option<TamagotchiState> = None;

#[no_mangle]
extern "C" fn init() {
    let current_block_height = exec::block_height() as u64;
    let tamagotchi = TamagotchiState {
        name: msg::load().expect("no name given"),
        date_of_birth: exec::block_timestamp(),
        owner: msg::source(),
        fed: tamagotchi_io::MIN_MOOD_VALUE,
        fed_block: current_block_height,
        entertained: tamagotchi_io::MIN_MOOD_VALUE,
        entertained_block: current_block_height,
        rested: tamagotchi_io::MIN_MOOD_VALUE,
        rested_block: current_block_height,
        allowed_account: None,
    };
    debug!("Tamagotchi info: {:?}", tamagotchi);
    unsafe {
        STATE = Some(tamagotchi);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: TmAction = msg::load().expect("no action given");
    let tamagotchi = unsafe { STATE.get_or_insert(Default::default()) };
    let current_block_height: u64 = exec::block_height() as u64;
    debug!("Block {:?}", current_block_height);
    match action {
        TmAction::Feed => {
            assert!(tamagotchi.verify_permission(msg::source()), "Only owner and allowed account can interact with this tamagotchi");
            tamagotchi.feed(current_block_height);
            msg::reply(TmEvent::Fed, 0).expect("reply failed on feed");
        }
        TmAction::Play => {
            assert!(tamagotchi.verify_permission(msg::source()), "Only owner and allowed account can interact with this tamagotchi");
            tamagotchi.play(current_block_height);
            msg::reply(TmEvent::Entertained, 0).expect("reply failed on entertain");
        }
        TmAction::Sleep => {
            assert!(tamagotchi.verify_permission(msg::source()), "Only owner and allowed account can interact with this tamagotchi");
            tamagotchi.sleep(current_block_height);
            msg::reply(TmEvent::Slept, 0).expect("reply failed on sleep");
        }
        TmAction::Name => {
            msg::reply(TmEvent::Name(tamagotchi.name.clone()), 0).expect("reply failed on name");
        }
        TmAction::Age => {
            msg::reply(
                TmEvent::Age(exec::block_timestamp() - tamagotchi.date_of_birth),
                0,
            )
            .expect("reply failed on age");
        }
        TmAction::Transfer(new_owner) => {
            assert!(tamagotchi.verify_ownership(msg::source()), "Only owner can transfer this tamagotchi");
            tamagotchi.owner = new_owner;
        }
        TmAction::Approve(allowed_account) => {
            assert!(tamagotchi.verify_ownership(msg::source()), "Only owner can approve an account");
            tamagotchi.allowed_account = Some(allowed_account);
        }
        TmAction::RevokeApproval => {
            assert!(tamagotchi.verify_ownership(msg::source()), "Only owner can revoke an account");
            tamagotchi.allowed_account = None;
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let tamagotchi = unsafe { STATE.as_ref().expect("tamagotchi not initialized") };
    msg::reply(tamagotchi, 0).expect("reply failed");
}

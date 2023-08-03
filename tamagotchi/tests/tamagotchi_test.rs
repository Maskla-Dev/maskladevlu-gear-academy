use gtest::{Program, System, Log};

use tamagotchi_io::{TamagotchiState, TmAction, TmEvent};

const TAMAGOTCHI_OWNER: u64 = 3;

#[test]
fn tamagotchi_initialization() {
    let sys = System::new();
    let program = Program::current(&sys);
    program.send(TAMAGOTCHI_OWNER, String::from("Armando"));
    let state: TamagotchiState = program.read_state().expect("File reading state");
    assert!(state.name == "Armando");
}

#[test]
fn tamagotchi_mood(){
    let sys = System::new();
    let program = Program::current(&sys);
    program.send(TAMAGOTCHI_OWNER, String::from("Armando"));
    let res = program.send(TAMAGOTCHI_OWNER, TmAction::Feed);
    assert!(!res.main_failed());
    let log = Log::builder().dest(TAMAGOTCHI_OWNER).payload(TmEvent::Fed);
    assert!(res.contains(&log));
    let res = program.send(TAMAGOTCHI_OWNER, TmAction::Sleep);
    assert!(!res.main_failed());
    let log = Log::builder().dest(TAMAGOTCHI_OWNER).payload(TmEvent::Slept);
    assert!(res.contains(&log));
    let res = program.send(TAMAGOTCHI_OWNER, TmAction::Play);
    assert!(!res.main_failed());
    let log = Log::builder().dest(TAMAGOTCHI_OWNER).payload(TmEvent::Entertained);
    assert!(res.contains(&log));
}
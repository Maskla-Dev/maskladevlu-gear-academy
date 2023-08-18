use gtest::{Program, System, Log};
use tamagotchi_io::{TamagotchiState, TmAction, TmEvent};

const FERNANDO: u64 = 3;
const LUIS: u64 = 4;

const TAMAGOTCHI: u64 = 1;

#[test]
fn tamagotchi_initialization() {
    let sys = System::new();
    init_tamagotchi(&sys);
    let program = sys.get_program(TAMAGOTCHI);
    let state: TamagotchiState = program.read_state().expect("File reading state");
    assert!(state.name == "Armando");
}

#[test]
fn tamagotchi_mood(){
    let sys = System::new();
    init_tamagotchi(&sys);
    let program = sys.get_program(TAMAGOTCHI);
    let res = program.send(FERNANDO, TmAction::Feed);
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Fed);
    assert!(res.contains(&log));
    let res = program.send(FERNANDO, TmAction::Sleep);
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Slept);
    assert!(res.contains(&log));
    let res = program.send(FERNANDO, TmAction::Play);
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Entertained);
    assert!(res.contains(&log));
}

#[test]
fn tamagotchi_nft(){
    let sys = System::new();
    init_tamagotchi(&sys);
    let program = sys.get_program(TAMAGOTCHI);
    //Verify ownership
    let res = program.send(LUIS, TmAction::Feed);
    assert!(res.main_failed());
    //Verify transfer
    let res = program.send(FERNANDO, TmAction::Transfer(LUIS.into()));
    assert!(!res.main_failed());
    let res = program.send(LUIS, TmAction::Feed);
    assert!(!res.main_failed());
    //Verify approval
    let res = program.send(LUIS, TmAction::Approve(FERNANDO.into()));
    assert!(!res.main_failed());
    let res = program.send(FERNANDO, TmAction::Feed);
    assert!(!res.main_failed());
    //Verify revocation
    let res = program.send(LUIS, TmAction::RevokeApproval);
    assert!(!res.main_failed());
    let res = program.send(FERNANDO, TmAction::Feed);
    assert!(res.main_failed());
}

fn init_tamagotchi(sys: &System) {
    let program = Program::current(&sys);
    let res = program.send(FERNANDO, String::from("Armando"));
    assert!(!res.main_failed());
}
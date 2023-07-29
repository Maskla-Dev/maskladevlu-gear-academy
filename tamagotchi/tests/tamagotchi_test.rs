use gtest::{Program, System};
use tamagotchi_io::TamagotchiState;

#[test]
fn tamagotchi_initialization() {
    let sys = System::new();
    let program = Program::current(&sys);
    program.send(3, String::from("Armando"));
    let state: TamagotchiState = program.read_state().expect("File reading state");
    println!("Current tamagotchi {:?}", state);
    assert!(state.name == "Armando");
}

use gtest::{Log, Program, System};
use hello_world::InputMessages;

#[test]
fn hello_test(){
    let sys = System::new();
    sys.init_logger();
    let program = Program::current(&sys);
    let res = program.send(2, String::from("Hello"));
    assert!(!res.main_failed());

    let hello_receipt: u64 = 4;
    let res = program.send(
        2,
        InputMessages::SendHelloTo(hello_receipt.into())
    );
    let expected_log = Log::builder().dest(hello_receipt).payload(String::from("Hello"));
    assert!(res.contains(&expected_log));
    let state: String = program.read_state().expect("Failed to read state");
    println!("State: {:?}", state);
}
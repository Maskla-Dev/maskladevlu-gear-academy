#![no_std]
use gstd::{debug, msg, prelude::*, ActorId, Decode, Encode, TypeInfo, Debug};

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum InputMessages {
    SendHelloTo(ActorId),
    SendHelloReply,
}

static mut GREETING: Option<String> = None;

#[no_mangle]
extern "C" fn init() {
    let greeting: String = msg::load().expect("Unable to decode message");
    debug!("Greeting: {}", greeting);
    unsafe {
        GREETING = Some(greeting);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let input_message = msg::load::<InputMessages>().expect("Unable to decode message");
    let greeting = unsafe { GREETING.as_mut().expect("Greeting not set") };
    match input_message {
        InputMessages::SendHelloTo(actor_id) => {
            msg::send(actor_id, greeting, 0).expect("Unable to send message");
        }
        InputMessages::SendHelloReply => {
            msg::reply(greeting, 0).expect("Unable to reply message");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let greeting = unsafe { GREETING.as_mut().expect("Greeting not set") };
    msg::reply(greeting, 0).expect("Failed to share state");
}

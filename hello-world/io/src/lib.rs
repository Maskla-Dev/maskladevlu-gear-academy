#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId, Encode, Decode, TypeInfo};
pub struct ProgramMetadata;

impl Metadata for ProgramMetadata{
    type Init = In<String>;
    type Handle = InOut<InputMessages, String>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = String;
}

#[derive(Encode, Decode, TypeInfo)]
pub enum InputMessages {
    SendHelloTo(ActorId),
    SendHelloReply
}
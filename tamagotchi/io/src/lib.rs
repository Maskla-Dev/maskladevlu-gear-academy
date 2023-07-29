#![no_std]
use gmeta::{Metadata, InOut, In};
use gstd::{prelude::*, Decode, Encode, TypeInfo, Debug};

pub struct TamagotchiMetadata; 

impl Metadata for TamagotchiMetadata{
    type Init = In<String>;
    type Handle = InOut<TmAction, TmEvent>;
    type Signal = ();
    type Reply = ();
    type Others = ();
    type State = TamagotchiState;
}

#[derive(Default, Encode, Decode, TypeInfo, Debug)]
pub struct TamagotchiState{
    pub name: String,
    pub date_of_birth: u64,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmAction{
    Name,
    Age
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmEvent{
    Name(String),
    Age(u64)
}
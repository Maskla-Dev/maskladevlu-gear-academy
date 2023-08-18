#![no_std]
use gmeta::metawasm;
use gstd::{prelude::*, ActorId};

#[metawasm]
pub mod metafns{
    pub type State = escrow_io::Escrow;

    pub fn seller(state: State) -> ActorId {
        state.seller
    }
    pub fn buyer(state: State) -> ActorId {
        state.buyer
    }
    pub fn price(state: State) -> escrow_io::EscrowState {
        state.state
    }
}
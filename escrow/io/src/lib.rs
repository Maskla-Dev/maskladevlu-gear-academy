#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId, Debug, Decode, Encode, Eq, PartialEq, TypeInfo, Default};

pub struct EscrowMetadata;

impl Metadata for EscrowMetadata {
    type Init = In<InitEscrow>;
    type Handle = InOut<EscrowAction, EscrowEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = EscrowState;
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub struct InitEscrow {
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum EscrowAction {
    Deposit(ActorId),
    ConfirmDelivery(ActorId),
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum EscrowEvent {
    ProgramInitialized,
    FundsDeposited,
    DeliveryConfirmed,
    PaymentToSeller,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode, TypeInfo)]
pub enum EscrowState {
    AwaitingPayment,
    AwaitingDelivery,
    Closed,
}

impl Default for EscrowState {
    fn default() -> Self {
        Self::AwaitingPayment
    }
}

#[derive(Default, Encode, Decode, TypeInfo, Debug)]
pub struct Escrow {
    pub factory_id: ActorId,
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
    pub state: EscrowState,
}

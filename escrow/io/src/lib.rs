#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{ActorId, Decode, Encode, TypeInfo, Debug, PartialEq, Default};

pub struct EscrowMetadata;

impl Metadata for EscrowMetadata {
    type Init = In<InitEscrow>;
    type Handle = InOut<EscrowAction, EscrowEvent>;
    type State = Escrow;
    type Others = ();
    type Reply = ();
    type Signal = ();
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub struct InitEscrow{
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowAction {
    Deposit,
    ConfirmDelivery
}

#[derive(Encode, Decode, TypeInfo, PartialEq, Debug, Default)]
pub enum EscrowState {
    #[default]
    AwaitingPayment,
    AwaitingDelivery,
    Closed,
}

#[derive(Encode, Decode, TypeInfo, Default, Debug)]
pub struct Escrow {
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
    pub state: EscrowState,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum EscrowEvent {
    FundsDeposited,
    DeliveryConfirmed,
}

impl Escrow {
    pub fn deposit(&mut self, source: &ActorId, amount: u128) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingPayment,
            "State must be 'Awaiting payment'"
        );
        assert_eq!(*source, self.buyer, "The message sender must be the buyer");
        assert_eq!(
            amount, self.price,
            "The attached value must be equal to set price"
        );
        self.state = EscrowState::AwaitingDelivery;
    }
    pub fn confirm_delivery(&mut self) {}
}

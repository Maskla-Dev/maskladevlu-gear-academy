#![no_std]
use escrow_io::{EscrowAction, EscrowEvent, EscrowState, InitEscrow};
use gstd::{msg, prelude::*, ActorId, debug};
static mut ESCROW: Option<Escrow> = None;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct Escrow {
    pub factory_id: ActorId,
    pub seller: ActorId,
    pub buyer: ActorId,
    pub price: u128,
    pub state: EscrowState,
}

impl Escrow {
    fn deposit(&mut self, account: &ActorId) {
        assert_eq!(
            self.state,
            EscrowState::AwaitingPayment,
            "State must be `AwaitingPayment"
        );

        assert_eq!(
            msg::source(),
            self.factory_id,
            "The message sender must be a factory contract"
        );

        assert_eq!(
            account, &self.buyer,
            "The indicated account must be a buyer"
        );

        assert_eq!(
            msg::value(),
            self.price,
            "The attached value must be equal to set price"
        );

        self.state = EscrowState::AwaitingDelivery;
        msg::reply(EscrowEvent::FundsDeposited, 0)
            .expect("Error in reply `EscrowEvent::FundsDeposited");
    }

    fn confirm_delivery(&mut self, account: &ActorId) {
        debug!("Confirm delivery...");
        assert_eq!(
            self.state,
            EscrowState::AwaitingDelivery,
            "State must be `AwaitingDelivery"
        );
        assert_eq!(
            msg::source(),
            self.factory_id,
            "The message sender must be a factory contract"
        );
        assert_eq!(
            account, &self.buyer,
            "The indicated account must be a buyer"
        );
        self.state = EscrowState::Closed;
        msg::send_with_gas(self.seller, EscrowEvent::PaymentToSeller, 0, self.price)
            .expect("Error in sending funds to the seller");
        msg::reply(EscrowEvent::DeliveryConfirmed, 0).expect("Error during a reply `FactoryEvent`");
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: EscrowAction = msg::load().expect("Unable to decode `EscrowAction`");
    let escrow = unsafe { ESCROW.as_mut().expect("Program hasn't been initialized") };
    match action {
        EscrowAction::Deposit(account) => escrow.deposit(&account),
        EscrowAction::ConfirmDelivery(account) => escrow.confirm_delivery(&account),
    }
}

#[no_mangle]
extern "C" fn init() {
    let InitEscrow {
        seller,
        buyer,
        price,
    } = msg::load().expect("Error in decoding `InitEscrow`");

    let escrow = Escrow {
        factory_id: msg::source(),
        seller,
        buyer,
        price,
        state: EscrowState::AwaitingPayment,
    };
    unsafe { ESCROW = Some(escrow) };

    msg::reply(EscrowEvent::ProgramInitialized, 0)
        .expect("Error during a reply `EscrowEvent::ProgramInitialized`");
}
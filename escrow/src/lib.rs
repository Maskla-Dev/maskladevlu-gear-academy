#![no_std]
use gstd::{debug, msg, prelude::*};

static mut ESCROW: Option<escrow_io::Escrow> = None;

#[no_mangle]
extern "C" fn init() {
    let init_escrow: escrow_io::InitEscrow =
        msg::load().expect("Cannot handle decode for init msg");
    debug!("Init escrow {:?}", init_escrow);
    let escrow = escrow_io::Escrow {
        seller: init_escrow.seller,
        buyer: init_escrow.buyer,
        price: init_escrow.price,
        state: escrow_io::EscrowState::AwaitingPayment,
    };
    unsafe {
        ESCROW = Some(escrow);
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: escrow_io::EscrowAction = msg::load().expect("Cannot handle decode for handle msg");
    let escrow: &mut escrow_io::Escrow =
        unsafe { ESCROW.as_mut().expect("The contract is not initialized") };
    let source = msg::source();
    match action {
        escrow_io::EscrowAction::Deposit => {
            escrow.deposit(&source, msg::value());
            msg::reply(escrow_io::EscrowEvent::FundsDeposited, 0)
                .expect("Error sending reply EscrowEvent::FundsDeposited");
        }
        escrow_io::EscrowAction::ConfirmDelivery => {
            debug!("Confirm delivery");
            escrow.confirm_delivery(&source);
            debug!("Sending funds to seller after confirming");
            msg::send(
                escrow.seller,
                escrow_io::EscrowEvent::DeliveryConfirmed,
                escrow.price,
            )
            .expect("Error sending funds to seller");
            debug!("Replying to buyer");
            msg::reply(escrow_io::EscrowEvent::DeliveryConfirmed, 0).expect("Cannot send reply");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let escrow = unsafe { ESCROW.get_or_insert(Default::default()) };
    msg::reply(escrow, 0).expect("Failed to share state");
}

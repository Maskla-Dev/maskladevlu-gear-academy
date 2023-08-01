#![no_std]
use gstd::{msg, prelude::*, debug};

static mut ESCROW: Option<escrow_io::Escrow> = None;

#[no_mangle]
extern "C" fn init() {
    let init_escrow: escrow_io::InitEscrow = msg::load().expect("Cannot handle decode for init msg");
    debug!("init escrow {:?}", init_escrow);
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
    let action: escrow_io::EscrowAction =
        msg::load().expect("Cannot handle decode for handle msg");
    let escrow: &mut escrow_io::Escrow = unsafe { ESCROW.as_mut().expect("The contract is not initialized") };
    let source = msg::source();
    match action {
        escrow_io::EscrowAction::Deposit => {
            escrow.deposit(&source, msg::value());
            msg::reply(escrow_io::EscrowEvent::FundsDeposited, 0).expect("Error sending reply EscrowEvent::FundsDeposited");
        },
        escrow_io::EscrowAction::ConfirmDelivery => escrow.confirm_delivery(),
    }
}

#[no_mangle]
extern "C" fn state() {
    let escrow = unsafe{ 
        ESCROW.get_or_insert(Default::default())
    };
    msg::reply(escrow, 0).expect("Failed to share state");
}

#![no_std]
use escrow_factory_io::{EscrowFactory, FactoryAction};
use gstd::{msg, prelude::*, CodeId, debug};

static mut STATE: Option<EscrowFactory> = None;

#[gstd::async_main]
async fn main() {
    let action: FactoryAction = msg::load().expect("Cannot decode action message");
    debug!("Action: {:?}", action);
    let factory = unsafe {
        STATE.get_or_insert(Default::default())
    };
    debug!("Current state: {:?}", factory);
    debug!("Source: {:?}", msg::source());
    match action {
        FactoryAction::CreateEscrow {
            seller,
            buyer,
            price,
        } => {
            factory.create_escrow(&seller, &buyer, price).await;
        }
        FactoryAction::Deposit(escrow_id) => {
            debug!("Deposit...");
            factory.deposit(escrow_id, &msg::source()).await;
        }
        FactoryAction::ConfirmDelivery(escrow_id) => {
            debug!("ConfirmDelivery...");
            factory.confirm_delivery(escrow_id, &msg::source()).await;
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let escrow_code_id: CodeId = msg::load().expect("Cannot decode init message");
    let escrow_factory = EscrowFactory {
        escrow_code_id,
        ..Default::default()
    };
    debug!("Escrow factory initialized with {:?}", escrow_factory);
    unsafe {
        STATE = Some(escrow_factory);
    }
}

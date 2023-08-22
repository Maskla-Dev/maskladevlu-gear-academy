use escrow_factory_io::{EscrowId, FactoryAction, FactoryEvent};
use gtest::{Log, Program, System};

const BUYER: u64 = 100;
const SELLER: u64 = 101;
const PRICE: u128 = 100_000;
const MASTER: u64 = 102;
const BUYER1: u64 = 103;
const SELLER1: u64 = 104;

#[test]
fn init_escrow_factory() {
    let system = System::new();
    system.init_logger();
    let escrow_code_id = system.submit_code(
        "/workspaces/maskladevlu-gear-academy/target/wasm32-unknown-unknown/debug/escrow.opt.wasm",
    );
    let escrow_factory = Program::current(&system);
    let res = escrow_factory.send(MASTER, escrow_code_id);
    assert!(!res.main_failed());
    //Creating Escrows
    let escrow0: EscrowId = 1;
    let escrow1: EscrowId = 2;
    let res = escrow_factory.send(
        MASTER,
        FactoryAction::CreateEscrow {
            seller: SELLER.into(),
            buyer: BUYER.into(),
            price: PRICE,
        },
    );
    assert!(!res.main_failed());
    let res = escrow_factory.send(
        MASTER,
        FactoryAction::CreateEscrow {
            seller: SELLER1.into(),
            buyer: BUYER1.into(),
            price: PRICE,
        },
    );
    assert!(!res.main_failed());
    //Escrow deposit
    system.mint_to(MASTER, PRICE);
    system.mint_to(BUYER, PRICE);
    system.mint_to(BUYER1, PRICE);
    system.mint_to(SELLER, PRICE);
    system.mint_to(SELLER1, PRICE);
    let res = escrow_factory.send_with_value(MASTER, FactoryAction::Deposit(escrow0), PRICE);
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(BUYER)
        .payload(FactoryEvent::Deposited(escrow0));
    assert!(res.contains(&log));
    let res = escrow_factory.send_with_value(MASTER, FactoryAction::Deposit(escrow1), PRICE);
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(BUYER1)
        .payload(FactoryEvent::Deposited(escrow1));
    assert!(res.contains(&log));
    //Escrow confirm delivery
    let res = escrow_factory.send_with_value(MASTER, FactoryAction::ConfirmDelivery(escrow0), PRICE);
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(SELLER)
        .payload(FactoryEvent::DeliveryConfirmed(escrow0));
    assert!(res.contains(&log));
    let res = escrow_factory.send_with_value(MASTER, FactoryAction::ConfirmDelivery(escrow1), PRICE);
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(SELLER1)
        .payload(FactoryEvent::DeliveryConfirmed(escrow1));
    assert!(res.contains(&log));
}

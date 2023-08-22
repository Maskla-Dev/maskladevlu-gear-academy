use ft_main_io::*;
use gstd::{ActorId, CodeId};
use gtest::{Log, Program, System};
use store_io::*;
use tamagotchi_factory_io::*;
use tamagotchi_io::*;

//Services
const SERVICE_MASTER: u64 = 102;
const FACTORY: u64 = 1;
const ATTRIBUTE_STORE: u64 = 5;
const FT_MAIN: u64 = 4;

//Raw path to contracts (only in github codespaces, replace with your own path)
const FT_STORAGE_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_storage.opt.wasm";
const FT_LOGIC_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_logic.opt.wasm";
const FT_MAIN_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_main.opt.wasm";
const STORE_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/store.opt.wasm";
const TAMAGOTCHI_FILE: &str =
    "/workspaces/maskladevlu-gear-academy/target/wasm32-unknown-unknown/debug/tamagotchi.opt.wasm";

//Actors
const FERNANDO: u64 = 100;
const LUIS: u64 = 101;

//Tamagotchis
const ARMANDILLO: u64 = 1;

//Transactions
const TRANSACTION_ID: u64 = 0;

#[test]
fn factory_initialization() {
    let system = System::new();
    system.init_logger();
    let ft_main = init_ft_main(&system);
    let tamagotchi_factory = init_factory(&system);
}

#[test]
fn tamagotchi_creation() {
    let system = System::new();
    system.init_logger();
    let ft_main = init_ft_main(&system);
    let tamagotchi_factory = init_factory(&system);
    create_tamagotchi(&tamagotchi_factory, FERNANDO, String::from("Armandillo"));
}

#[test]
fn tamagotchi_mood() {
    let system = System::new();
    system.init_logger();
    let ft_main = init_ft_main(&system);
    let tamagotchi_factory = init_factory(&system);
    create_tamagotchi(&tamagotchi_factory, FERNANDO, String::from("Armandillo"));
    let res = tamagotchi_factory.send(FERNANDO, TMFAction::TmAction(1, _TmAction::Feed));
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Fed);
    assert!(res.contains(&log));
    let res = tamagotchi_factory.send(FERNANDO, TMFAction::TmAction(1, _TmAction::Sleep));
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Slept);
    assert!(res.contains(&log));
    let res = tamagotchi_factory.send(FERNANDO, TMFAction::TmAction(1, _TmAction::Play));
    assert!(!res.main_failed());
    let log = Log::builder().dest(FERNANDO).payload(TmEvent::Entertained);
    assert!(res.contains(&log));
}

#[test]
fn tamagotchi_nft() {
    let sys = System::new();
    sys.init_logger();
    let ft_main = init_ft_main(&sys);
    let tamagotchi_factory = init_factory(&sys);
    create_tamagotchi(&tamagotchi_factory, FERNANDO, String::from("Armandillo"));
    //Verify ownership
    let res = tamagotchi_factory.send(LUIS, TMFAction::TmAction(1, _TmAction::Feed));
    assert!(res.main_failed());
    //Verify transfer
    let res = tamagotchi_factory.send(
        FERNANDO,
        TMFAction::TmAction(ARMANDILLO, _TmAction::Transfer(LUIS.into())),
    );
    assert!(!res.main_failed());
    let res = tamagotchi_factory.send(LUIS, TMFAction::TmAction(1, _TmAction::Feed));
    assert!(!res.main_failed());
    //Verify approval
    let res = tamagotchi_factory.send(
        LUIS,
        TMFAction::TmAction(ARMANDILLO, _TmAction::Approve(FERNANDO.into())),
    );
    assert!(!res.main_failed());
    let res = tamagotchi_factory.send(FERNANDO, TMFAction::TmAction(1, _TmAction::Feed));
    assert!(!res.main_failed());
    //Verify revocation
    let res = tamagotchi_factory.send(
        LUIS,
        TMFAction::TmAction(ARMANDILLO, _TmAction::RevokeApproval),
    );
    assert!(!res.main_failed());
    let res = tamagotchi_factory.send(FERNANDO, TMFAction::TmAction(ARMANDILLO, _TmAction::Feed));
    assert!(res.main_failed());
}

#[test]
fn tamagotchi_attribute_purchase() {
    let sys = System::new();
    let store = init_store(&sys);
    let ft_main = init_ft_main(&sys);
    let factory = init_factory(&sys);
    sys.init_logger();
    create_tamagotchi(&factory, FERNANDO, String::from("Armandillo"));
    mint_tokens_for(&ft_main, LUIS.into(), 1000);
    check_balance(&ft_main, LUIS, 1000);
    mint_tokens_for(&ft_main, FERNANDO.into(), 500);
    check_balance(&ft_main, FERNANDO, 500);
    mint_tokens_for(&ft_main, ATTRIBUTE_STORE.into(), 100);
    check_balance(&ft_main, ATTRIBUTE_STORE, 100);
    let superbomba: AttrMetadata = AttrMetadata {
        title: String::from("Superbomba"),
        description: String::from("Superbomba atribute"),
        media: String::from("www.nowhere.com/Superbomba"),
    };
    let superbomba_id: AttributeId = 123;
    create_attribute(&store, &superbomba, 1000, superbomba_id);
    let res = factory.send(
        FERNANDO,
        TMFAction::TmAction(ARMANDILLO, _TmAction::SetTokenContract(FT_MAIN.into())),
    );
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(FERNANDO)
        .payload(TmEvent::TokenContractSet);
    assert!(res.contains(&log));
    //     //Expected flow
    println!("Approve tokens...");
    let res = factory.send(
        FERNANDO,
        TMFAction::TmAction(
            ARMANDILLO,
            _TmAction::ApproveTokens {
                account: ATTRIBUTE_STORE.into(),
                amount: 1000,
            },
        ),
    );
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(FERNANDO)
        .payload(TmEvent::TokensApproved {
            account: ATTRIBUTE_STORE.into(),
            amount: 1000,
        });
    assert!(res.contains(&log));
    println!("Tokens successfully approved...");
    println!("Buying attribute...");
    let res = factory.send(
        FERNANDO,
        TMFAction::TmAction(
            ARMANDILLO,
            _TmAction::BuyAttribute {
                store_id: ATTRIBUTE_STORE.into(),
                attribute_id: superbomba_id,
            },
        ),
    );
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(ATTRIBUTE_STORE)
        .payload(TmEvent::AttributeBought(superbomba_id));
    assert!(res.contains(&log));
    println!("Attribute successfully bought...");
}

//Helpers
fn init_factory(system: &System) -> Program {
    let tamagotchi_code_id = system.submit_code(TAMAGOTCHI_FILE);
    let tamagotchi_factory = Program::current(&system);
    let res = tamagotchi_factory.send(
        SERVICE_MASTER,
        InitTamagotchiFactory {
            tamagotchi_bank: FT_MAIN.into(),
            tamagotchi_template: CodeId::from(tamagotchi_code_id.into_bytes()),
        },
    );
    assert!(!res.main_failed());
    tamagotchi_factory
}

fn init_store(sys: &System) -> Program {
    let program = Program::from_file_with_id(&sys, ATTRIBUTE_STORE, STORE_FILE);
    let res = program.send::<_, ActorId>(SERVICE_MASTER, FT_MAIN.into());
    assert!(!res.main_failed());
    // println!("Successfully Store contract loaded: {:?}", res);
    program
}

fn init_ft_main(system: &System) -> Program {
    let program = Program::from_file_with_id(&system, FT_MAIN, FT_MAIN_FILE);
    let storage_code_hash: [u8; 32] = system.submit_code(FT_STORAGE_FILE).into();
    let logic_code_hash: [u8; 32] = system.submit_code(FT_LOGIC_FILE).into();
    let res = program.send(
        SERVICE_MASTER,
        InitFToken {
            ft_logic_code_hash: logic_code_hash.into(),
            storage_code_hash: storage_code_hash.into(),
        },
    );
    assert!(!res.main_failed());
    // println!("Successfully FT contract loaded: {:?}", res);
    program
}

fn create_tamagotchi(tamagotchi_factory: &Program, actor: u64, name: String) {
    let res = tamagotchi_factory.send(actor, TMFAction::CreateTamagotchi(name));
    assert!(!res.main_failed());
}

fn mint_tokens_for(ft_program: &Program, recipient: ActorId, amount: u128) {
    let result = ft_program.send(
        SERVICE_MASTER,
        FTokenAction::Message {
            transaction_id: TRANSACTION_ID,
            payload: LogicAction::Mint { recipient, amount },
        },
    );
    let log = Log::builder().dest(SERVICE_MASTER).payload(FTokenEvent::Ok);
    assert!(!result.main_failed());
    assert!(result.contains(&log));
}

fn check_balance(ft_program: &Program, account: impl Into<ActorId>, expected_amount: u128) {
    let res = ft_program.send(SERVICE_MASTER, FTokenAction::GetBalance(account.into()));
    let payload = Log::builder()
        .dest(SERVICE_MASTER)
        .payload(FTokenEvent::Balance(expected_amount));
    assert!(res.contains(&payload));
}

fn create_attribute(
    store_program: &Program,
    attr_metadata: &AttrMetadata,
    price: u128,
    attribute_id: AttributeId,
) {
    let res = store_program.send(
        SERVICE_MASTER,
        StoreAction::CreateAttribute {
            attribute_id: attribute_id,
            attr_metadata: attr_metadata.clone(),
            price: price,
        },
    );
    assert!(!res.main_failed());
    let log = Log::builder()
        .dest(SERVICE_MASTER)
        .payload(StoreEvent::AttributeCreated { attribute_id });
    assert!(res.contains(&log));
}

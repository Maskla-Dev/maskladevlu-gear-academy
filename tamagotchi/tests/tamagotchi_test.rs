// use ft_main_io::InitFToken;
// use ft_main_io::*;
// use gstd::{prelude::*, ActorId};
// use gtest::{Log, Program, System};
// use store_io::*;
// use tamagotchi_io::{TamagotchiState, TmAction, TmEvent};

// //Codespaces only, replace with your source from dapps-smart-contract-academy/contracts/upload-contracts/programs/
// const FT_STORAGE_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_storage.opt.wasm";
// const FT_LOGIC_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_logic.opt.wasm";
// const FT_MAIN_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/ft_main.opt.wasm";
// const STORE_FILE: &str = "/workspaces/maskladevlu-gear-academy/contracts/store.opt.wasm";

// #[test]
// fn tamagotchi_initialization() {
//     let sys = System::new();
//     let program = init_tamagotchi(&sys);
//     // let program = sys.get_program(TAMAGOTCHI);
//     let state: TamagotchiState = program.read_state().expect("File reading state");
//     assert!(state.name == "Armando");
// }

// #[test]
// fn tamagotchi_mood() {
//     let sys = System::new();
//     let program = init_tamagotchi(&sys);
//     // let program = sys.get_program(TAMAGOTCHI);
//     let res = program.send(FERNANDO, TmAction::Feed);
//     assert!(!res.main_failed());
//     let log = Log::builder().dest(FERNANDO).payload(TmEvent::Fed);
//     assert!(res.contains(&log));
//     let res = program.send(FERNANDO, TmAction::Sleep);
//     assert!(!res.main_failed());
//     let log = Log::builder().dest(FERNANDO).payload(TmEvent::Slept);
//     assert!(res.contains(&log));
//     let res = program.send(FERNANDO, TmAction::Play);
//     assert!(!res.main_failed());
//     let log = Log::builder().dest(FERNANDO).payload(TmEvent::Entertained);
//     assert!(res.contains(&log));
// }

// #[test]
// fn tamagotchi_nft() {
//     let sys = System::new();
//     let program = init_tamagotchi(&sys);
//     // let program = sys.get_program(TAMAGOTCHI);
//     //Verify ownership
//     let res = program.send(LUIS, TmAction::Feed);
//     assert!(res.main_failed());
//     //Verify transfer
//     let res = program.send(FERNANDO, TmAction::Transfer(LUIS.into()));
//     assert!(!res.main_failed());
//     let res = program.send(LUIS, TmAction::Feed);
//     assert!(!res.main_failed());
//     //Verify approval
//     let res = program.send(LUIS, TmAction::Approve(FERNANDO.into()));
//     assert!(!res.main_failed());
//     let res = program.send(FERNANDO, TmAction::Feed);
//     assert!(!res.main_failed());
//     //Verify revocation
//     let res = program.send(LUIS, TmAction::RevokeApproval);
//     assert!(!res.main_failed());
//     let res = program.send(FERNANDO, TmAction::Feed);
//     assert!(res.main_failed());
// }

// #[test]
// fn tamagotchi_attribute_purchase() {
//     let sys = System::new();
//     let store = init_store(&sys);
//     let ft_main = init_ft_main(&sys);
//     let tamagotchi = init_tamagotchi(&sys);
//     sys.init_logger();
//     mint_tokens_for(&ft_main, LUIS.into(), 1000);
//     check_balance(&ft_main, LUIS, 1000);
//     mint_tokens_for(&ft_main, FERNANDO.into(), 500);
//     check_balance(&ft_main, FERNANDO, 500);
//     mint_tokens_for(&ft_main, ATTRIBUTE_STORE.into(), 100);
//     check_balance(&ft_main, ATTRIBUTE_STORE, 100);
//     let superbomba: AttrMetadata = AttrMetadata {
//         title: String::from("Superbomba"),
//         description: String::from("Superbomba atribute"),
//         media: String::from("www.nowhere.com/Superbomba"),
//     };
//     let superbomba_id: AttributeId = 123;
//     create_attribute(&store, &superbomba, 1000, superbomba_id);
//     let res = tamagotchi.send(FERNANDO, TmAction::SetTokenContract(FT_MAIN.into()));
//     assert!(!res.main_failed());
//     let log = Log::builder()
//         .dest(FERNANDO)
//         .payload(TmEvent::TokenContractSet);
//     assert!(res.contains(&log));
//     //Expected flow
//     println!("Approve tokens...");
//     let res = tamagotchi.send(
//         FERNANDO,
//         TmAction::ApproveTokens {
//             account: ATTRIBUTE_STORE.into(),
//             amount: 1000,
//         },
//     );
//     assert!(!res.main_failed());
//     let log = Log::builder()
//         .dest(FERNANDO)
//         .payload(TmEvent::TokensApproved {
//             account: ATTRIBUTE_STORE.into(),
//             amount: 1000,
//         });
//     assert!(res.contains(&log));
//     println!("Tokens successfully approved...");
//     println!("Buying attribute...");
//     let res = tamagotchi.send(
//         FERNANDO,
//         TmAction::BuyAttribute {
//             store_id: ATTRIBUTE_STORE.into(),
//             attribute_id: superbomba_id,
//         },
//     );
//     assert!(!res.main_failed());
//     let log = Log::builder()
//         .dest(ATTRIBUTE_STORE)
//         .payload(TmEvent::AttributeBought(superbomba_id));
//     assert!(res.contains(&log));
//     println!("Attribute successfully bought...");
// }

// const FERNANDO: u64 = 100;
// const LUIS: u64 = 101;
// const SERVICE_MASTER: u64 = 6;

// const TAMAGOTCHI: u64 = 1;
// const FT_STORAGE: u64 = 2;
// const FT_LOGIC: u64 = 3;
// const FT_MAIN: u64 = 4;
// const ATTRIBUTE_STORE: u64 = 5;

// const TRANSACTION_ID: u64 = 0;

// fn init_tamagotchi(sys: &System) -> Program {
//     let program = Program::current_with_id(&sys, TAMAGOTCHI);
//     let res = program.send(FERNANDO, String::from("Armando"));
//     assert!(!res.main_failed());
//     // println!("Successfully Tamagotchi contract loaded: {:?}", res);
//     program
// }

// fn init_store(sys: &System) -> Program {
//     let program = Program::from_file_with_id(&sys, ATTRIBUTE_STORE, STORE_FILE);
//     let res = program.send::<_, ActorId>(SERVICE_MASTER, FT_MAIN.into());
//     assert!(!res.main_failed());
//     // println!("Successfully Store contract loaded: {:?}", res);
//     program
// }

// fn init_ft_main(system: &System) -> Program {
//     let program = Program::from_file_with_id(&system, FT_MAIN, FT_MAIN_FILE);
//     let storage_code_hash: [u8; 32] = system.submit_code(FT_STORAGE_FILE).into();
//     let logic_code_hash: [u8; 32] = system.submit_code(FT_LOGIC_FILE).into();
//     let res = program.send(
//         SERVICE_MASTER,
//         InitFToken {
//             ft_logic_code_hash: logic_code_hash.into(),
//             storage_code_hash: storage_code_hash.into(),
//         },
//     );
//     assert!(!res.main_failed());
//     // println!("Successfully FT contract loaded: {:?}", res);
//     program
// }

// fn mint_tokens_for(ft_program: &Program, recipient: ActorId, amount: u128) {
//     let result = ft_program.send(
//         SERVICE_MASTER,
//         FTokenAction::Message {
//             transaction_id: TRANSACTION_ID,
//             payload: LogicAction::Mint { recipient, amount },
//         },
//     );
//     let log = Log::builder().dest(SERVICE_MASTER).payload(FTokenEvent::Ok);
//     assert!(!result.main_failed());
//     assert!(result.contains(&log));
// }

// fn check_balance(ft_program: &Program, account: impl Into<ActorId>, expected_amount: u128) {
//     let res = ft_program.send(SERVICE_MASTER, FTokenAction::GetBalance(account.into()));
//     let payload = Log::builder()
//         .dest(SERVICE_MASTER)
//         .payload(FTokenEvent::Balance(expected_amount));
//     assert!(res.contains(&payload));
// }

// fn create_attribute(
//     store_program: &Program,
//     attr_metadata: &AttrMetadata,
//     price: u128,
//     attribute_id: AttributeId,
// ) {
//     let res = store_program.send(
//         SERVICE_MASTER,
//         StoreAction::CreateAttribute {
//             attribute_id: attribute_id,
//             attr_metadata: attr_metadata.clone(),
//             price: price,
//         },
//     );
//     assert!(!res.main_failed());
//     let log = Log::builder()
//         .dest(SERVICE_MASTER)
//         .payload(StoreEvent::AttributeCreated { attribute_id });
//     assert!(res.contains(&log));
// }

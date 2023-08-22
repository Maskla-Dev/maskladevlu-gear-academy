#![no_std]
use gstd::{debug, exec, msg, prelude::*};
use store_io::{StoreAction, StoreEvent};
use tamagotchi_io::{InitTamagotchi, TamagotchiState, TmAction, TmEvent, CHECK_INTERVAL};

static mut STATE: Option<TamagotchiState> = None;

#[no_mangle]
extern "C" fn init() {
    let current_block_height = exec::block_height() as u64;
    let tamagotchi_init: InitTamagotchi = msg::load().expect("No init data given");
    let tamagotchi = TamagotchiState {
        commander: tamagotchi_init.commander,
        name: tamagotchi_init.name,
        date_of_birth: exec::block_timestamp(),
        owner: tamagotchi_init.owner,
        fed: tamagotchi_io::MIN_MOOD_VALUE,
        fed_block: current_block_height,
        entertained: tamagotchi_io::MIN_MOOD_VALUE,
        entertained_block: current_block_height,
        rested: tamagotchi_io::MIN_MOOD_VALUE,
        rested_block: current_block_height,
        ..Default::default()
    };
    debug!("Tamagotchi info: {:?}", tamagotchi);
    unsafe {
        STATE = Some(tamagotchi);
    }
    msg::send_delayed(
        exec::program_id(),
        TmAction::CheckState(tamagotchi_init.owner),
        0,
        CHECK_INTERVAL,
    )
    .expect("Failed to send delayed in init");
    msg::reply(TmEvent::TamagotchiInitiliazed, 0).expect("reply failed on init");
}

#[gstd::async_main]
async fn main() {
    let action: TmAction = msg::load().expect("no action given");
    let tamagotchi = unsafe { STATE.get_or_insert(Default::default()) };
    let is_commander = msg::source() == tamagotchi.commander;

    let current_block_height: u64 = exec::block_height() as u64;
    tamagotchi.update_mood(current_block_height);
    debug!("Block {:?}", current_block_height);
    match action {
        TmAction::Feed(from) => {
            assert!(
                tamagotchi.verify_permission(from),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            tamagotchi.feed();
            msg::reply(TmEvent::Fed, 0).expect("reply failed on feed");
        }
        TmAction::Play(from) => {
            assert!(
                tamagotchi.verify_permission(from),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            tamagotchi.play();
            msg::reply(TmEvent::Entertained, 0).expect("reply failed on entertain");
        }
        TmAction::Sleep(from) => {
            assert!(
                tamagotchi.verify_permission(from),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            tamagotchi.sleep();
            msg::reply(TmEvent::Slept, 0).expect("reply failed on sleep");
        }
        TmAction::Name => {
            msg::reply(TmEvent::Name(tamagotchi.name.clone()), 0).expect("reply failed on name");
        }
        TmAction::Age => {
            msg::reply(
                TmEvent::Age(exec::block_timestamp() - tamagotchi.date_of_birth),
                0,
            )
            .expect("reply failed on age");
        }
        TmAction::Transfer {
            from,
            to: new_owner,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_permission(from),
                "Only owner can transfer this tamagotchi"
            );
            tamagotchi.owner = new_owner;
        }
        TmAction::Approve {
            from,
            to: allowed_account,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_ownership(from),
                "Only owner can approve an account"
            );
            tamagotchi.allowed_account = Some(allowed_account);
        }
        TmAction::RevokeApproval(from) => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_ownership(from),
                "Only owner can revoke an account"
            );
            tamagotchi.allowed_account = None;
        }
        TmAction::SetTokenContract {
            source,
            new_bank: ft_contract,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_ownership(source),
                "Only owner can set the token contract"
            );
            tamagotchi.ft_contract = Some(ft_contract);
            msg::reply(TmEvent::TokenContractSet, 0).expect("reply failed on set token contract");
        }
        TmAction::BuyAttribute {
            source: allowed,
            store_id,
            attribute_id,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_permission(allowed),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            let result = msg::send_for_reply_as::<_, StoreEvent>(
                store_id,
                StoreAction::BuyAttribute { attribute_id },
                0,
                0,
            )
            .expect("Failed to send buy attribute message")
            .await;
            debug!("Successfully sent buy attribute message");
            if let Ok(StoreEvent::AttributeSold { success }) = result {
                if success {
                    debug!("Attribute bought");
                    msg::reply(TmEvent::AttributeBought(attribute_id), 0)
                        .expect("reply failed on buy attribute");
                } else {
                    debug!("Attribute not bought");
                    msg::reply(TmEvent::ErrorDuringPurchase, 0)
                        .expect("reply failed on buy attribute");
                }
            } else {
                panic!("Unexpected reply from store");
            };
        }
        TmAction::ApproveTokens {
            source,
            account,
            amount,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_permission(source),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            debug!("Successfully verified permission");
            let approval_result = tamagotchi.approve_tokens(&account, amount).await;
            debug!("Successfully approved tokens");
            msg::reply(approval_result, 0).expect("Error sending approval result");
        }
        TmAction::Owner => {
            msg::reply(TmEvent::Owner(tamagotchi.owner), 0).expect("reply failed on owner");
        }
        TmAction::CheckState(source) => {
            assert!(
                tamagotchi.verify_permission(source.clone()),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            tamagotchi.check_state_flow(&source);
        }
        TmAction::ReserveGas {
            source,
            reservation_amount,
            duration,
        } => {
            assert!(
                is_commander,
                "Only commander can interact with this tamagotchi"
            );
            assert!(
                tamagotchi.verify_permission(source),
                "Only owner and allowed account can interact with this tamagotchi"
            );
            msg::reply(tamagotchi.make_reservation(reservation_amount, duration), 0)
                .expect("reply failed on reserve gas");
            msg::send_delayed(exec::program_id(), TmAction::CheckState(source), 0, CHECK_INTERVAL)
                .expect("Failed to send delayed in reserve gas");
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    let tamagotchi = unsafe { STATE.as_ref().expect("tamagotchi not initialized") };
    msg::reply(tamagotchi, 0).expect("reply failed");
}

#![no_std]
use gstd::{debug, msg, prelude::*, ext::debug};
use tamagotchi_factory_io::*;

static mut STATE: Option<TamagotchiFactory> = None;

#[gstd::async_main]
async fn main() {
    let action: TMFAction = msg::load().expect("Cannot decode action message");
    debug!("Action: {:?}", action);
    let factory = unsafe { STATE.get_or_insert(Default::default()) };
    match action {
        TMFAction::CreateTamagotchi(payload) => {
            debug!("Creating tamagotchi");
            factory.create_tamagotchi(payload).await;
        }
        TMFAction::TmAction(tamagotchi, payload) => {
            debug!("Processing tamagotchi action");
            factory
                .process_tm_action(tamagotchi, payload, &msg::source())
                .await;
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let init_params: InitTamagotchiFactory =
        msg::load().expect("Cannot get tamagotchi template code id");
    debug!("Tamagotchi init params: {:?}", init_params);
    unsafe {
        STATE = Some(TamagotchiFactory {
            tamagotchi_template: init_params.tamagotchi_template,
            tamagotchi_number: 0,
            id_to_address: BTreeMap::new(),
            tamagotchi_bank: init_params.tamagotchi_bank,
        });
    }
}

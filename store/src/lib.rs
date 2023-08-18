#![no_std]
use gstd::{async_main, msg, prelude::*, ActorId, Decode, Encode, TypeInfo};
use storage_io::*;

static mut STORAGE: Option<AttributeStorage> = None;

#[std::async_main]
async fn main() {
    let source = msg::source();
    let action: StorageAction = msg::load().expect("Failed to load storage action");
    let store: &mut AttributeStorage = unsafe { STORAGE.get_or_insert_with(Default::default()) };
    match action {
        StorageAction::CreateAttribute {
            attribute_id,
            metadata,
            price,
        } => {
            store.create_attribute(&source, attribute_id, &metadata, price);
            msg::reply(StorageEvent::AttributeCreated { attribute_id }, 0)
                .expect("Cannot send reply while creating attribute");
        }
        StorageAction::BuyAttribute { attribute_id } => {
            let result = store.buy_attribute(&source, attribute_id).await;
            msg::reply(result, 0).expect("Cannot send reply while buying attribute");
        }
    }
}

#[no_mangle]
extern "C" fn init() {
    let ft_contract_id = msg::load().expect("Failed to decode init message");
    let store = AttributeStorage {
        admin: msg::source(),
        ft_contract_id,
        ..Default::default()
    };
    unsafe {
        STORAGE = Some(store);
    }
}

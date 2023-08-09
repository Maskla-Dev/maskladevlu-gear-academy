#![no_std]
use ft_main_io::*;
use gmeta::{In, InOut, Metadata};
use gstd::{msg, prelude::*, ActorId, Clone, Decode, Default, Encode, TypeInfo};

pub type AttributeId = u64;
pub type Price = u128;
pub type TamagotchiId = ActorId;
pub type TransactionId = u64;

pub struct StoreMeta;

impl Metadata for StoreMeta {
    type Init = In<ActorId>;
    type Handle = InOut<StoreAction, StoreEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = ();
}

#[derive(Decode, Encode, TypeInfo, Clone)]
pub struct AttributeMetadata {
    pub title: String,
    pub description: String,
    pub media: String,
}

#[derive(Decode, Encode, TypeInfo)]
pub enum StoreAction {
    CreateAttribute {
        attribute_id: AttributeId,
        metadata: AttributeMetadata,
        price: Price,
    },
    BuyAttribute {
        attribute_id: AttributeId,
    },
}

#[derive(Decode, Encode, TypeInfo)]
pub enum StoreEvent {
    AttributeCreated { attribute_id: AttributeId },
    AttributeSold { success: bool },
    CompletePrevTask { attribute_id: AttributeId },
}

#[derive(Decode, Encode, TypeInfo, Default)]
pub struct AttributeStore {
    admin: ActorId,
    ft_contract_id: ActorId,
    attributes: BTreeMap<AttributeId, (AttributeMetadata, Price)>,
    owners: BTreeMap<TamagotchiId, BTreeSet<AttributeId>>,
    transaction_id: TransactionId,
    transactions: BTreeMap<TamagotchiId, (TransactionId, AttributeId)>,
}

impl AttributeStore {
    pub fn create_attribute(
        &mut self,
        source: &ActorId,
        attribute_id: AttributeId,
        metadata: &AttributeMetadata,
        price: Price,
    ) {
        assert_eq!(self.admin, *source, "Only admin can create attributes");
        if self
            .attributes
            .insert(attribute_id, (metadata.clone(), price))
            .is_some()
        {
            panic!("Attribute with that ID already exists");
        }
    }

    // This function is called when a user wants to buy an attribute
    // Prevents double spending by checking if the user has already started a transaction
    pub async fn buy_attribute(
        &mut self,
        source: &ActorId,
        program_id: &ActorId,
        attribute_id: AttributeId,
    ) -> StoreEvent {
        let (transaction_id, attribute_id) =
        // Get the transactions for the user <source: ActorId> If there is a transaction is in the mapping...
            if let Some((transaction_id, prev_attribute_id)) = self.transactions.get(source) {
                // If `prev_attribute_id` is not equal to `attribute_id`, it means the transaction wasn't completed
                // We'll ask the Tamagotchi contract to complete the previous transaction
                if attribute_id != *prev_attribute_id {
                    return StoreEvent::CompletePrevTask {
                        attribute_id: *prev_attribute_id,
                    }
                }
                // Otherwise, we'll just return the transaction_id and attribute_id
                (*transaction_id, *prev_attribute_id)
            } else {
                // when there is no transaction, we'll create a new transaction
                let current_transaction_id = self.transaction_id;
                self.transaction_id = self.transaction_id.wrapping_add(1);
                self.transactions
                    .insert(*source, (current_transaction_id, attribute_id));
                (current_transaction_id, attribute_id)
            };
        // Next, we'll call the `sell_attribute` function to sell the attribute
        let result = self
            .sell_attribute(source, program_id, transaction_id, attribute_id)
            .await;
        //After the transaction is complete, we'll remove the transaction from the mapping
        self.transactions.remove(source);
        //Successful transaction
        StoreEvent::AttributeSold { success: result }
    }

    async fn transfer_tokens(
        transaction_id: TransactionId,
        token_address: &ActorId,
        from: &ActorId,
        to: &ActorId,
        amount_tokens: u128,
    ) -> Result<(), ()> {
        let reply = msg::send_for_reply_as::<_, FTokenEvent>(
            *token_address,
            FTokenAction::Message {
                transaction_id,
                payload: LogicAction::Transfer {
                    sender: *from,
                    recipient: *to,
                    amount: amount_tokens,
                },
            },
            0,
            0,
        )
        .expect("Error in sending message 'FTokenAction::Message'")
        .await;
        match reply {
            Ok(FTokenEvent::Ok) => Ok(()),
            _ => Err(()),
        }
    }
    async fn sell_attribute(
        &mut self,
        source: &ActorId,
        program_id: &ActorId,
        transaction_id: TransactionId,
        attribute_id: AttributeId,
    ) -> bool {
        let (_, price) = self
            .attributes
            .get(&attribute_id)
            .expect("Attribute not found");
        if AttributeStore::transfer_tokens(
            transaction_id,
            &self.ft_contract_id,
            source,
            program_id,
            *price,
        )
        .await
        .is_ok()
        {
            self.owners
                .entry(*source)
                .and_modify(|attributes| {
                    attributes.insert(attribute_id);
                })
                .or_insert_with(|| [attribute_id].into());
            return true;
        }
        false
    }
}

#![no_std]
use escrow_io::{EscrowAction, EscrowEvent, InitEscrow};
use gmeta::{In, InOut, Metadata};
use gstd::{
    msg, prelude::*, prog::ProgramGenerator, ActorId, CodeId, Decode, Default, Encode, TypeInfo,
};

pub struct EscrowFactoryMetadata;

impl Metadata for EscrowFactoryMetadata {
    type Init = In<CodeId>;
    type Handle = InOut<FactoryAction, FactoryEvent>;
    type Reply = ();
    type Signal = ();
    type Others = ();
    type State = EscrowFactory;
}

pub type EscrowId = u64;
const GAS_FOR_CREATION: u64 = 100_000_000;

#[derive(Default, Encode, Decode, TypeInfo)]
pub struct EscrowFactory {
    pub escrow_number: EscrowId,
    pub id_to_address: BTreeMap<EscrowId, ActorId>,
    pub escrow_code_id: CodeId,
}

impl EscrowFactory {
    pub async fn create_escrow(&mut self, seller: &ActorId, buyer: &ActorId, price: u128) {
        let (address, _) = ProgramGenerator::create_program_with_gas_for_reply(
            self.escrow_code_id,
            InitEscrow {
                seller: *seller,
                buyer: *buyer,
                price,
            }
            .encode(),
            GAS_FOR_CREATION,
            0,
            0,
        )
        .expect("Error creating escrow")
        .await
        .expect("Program was not initialized");
        self.escrow_number = self.escrow_number.saturating_add(1);
        self.id_to_address.insert(self.escrow_number, address);
        msg::reply(
            FactoryEvent::EscrowCreated {
                escrow_id: self.escrow_number,
                escrow_address: address,
            },
            0,
        )
        .expect("Error replying at: finishing escrow creation");
    }

    pub async fn deposit(&self, escrow_id: EscrowId, source: &ActorId) {
        let escrow_address = self.get_escrow_address(escrow_id);
        send_message(&escrow_address, EscrowAction::Deposit(*source)).await;
        msg::reply(FactoryEvent::Deposited(escrow_id), 0)
            .expect("Error replying at: finishing deposit");
    }

    pub async fn confirm_delivery(&self, escrow_id: EscrowId, source: &ActorId) {
        let escrow_address = self.get_escrow_address(escrow_id);
        send_message(&escrow_address, EscrowAction::ConfirmDelivery(*source)).await;
        msg::reply(FactoryEvent::DeliveryConfirmed(escrow_id), 0)
            .expect("Error replying at: finishing delivery confirmation");
    }

    fn get_escrow_address(&self, escrow_id: EscrowId) -> ActorId {
        *self
            .id_to_address
            .get(&escrow_id)
            .expect("Escrow does not exist")
    }
}

async fn send_message(escrow_address: &ActorId, action: EscrowAction) {
    msg::send_for_reply_as::<_, EscrowEvent>(escrow_address.clone(), action, msg::value(), 0)
        .expect("Error sending message")
        .await
        .expect("Unnable to decode EscrowEvent");
}

#[derive(Encode, Decode, TypeInfo)]
pub enum FactoryAction {
    CreateEscrow {
        seller: ActorId,
        buyer: ActorId,
        price: u128,
    },
    Deposit(EscrowId),
    ConfirmDelivery(EscrowId),
}

#[derive(Encode, Decode, TypeInfo)]
pub enum FactoryEvent {
    EscrowCreated {
        escrow_id: EscrowId,
        escrow_address: ActorId,
    },
    Deposited(EscrowId),
    DeliveryConfirmed(EscrowId),
}

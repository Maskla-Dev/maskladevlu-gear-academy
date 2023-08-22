#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{
    debug, exec, ext::debug, msg, prelude::*, prog::ProgramGenerator, ActorId, CodeId, Debug,
    Decode, Default, Encode, TypeInfo,
};
use store_io::AttributeId;
use tamagotchi_io::{InitTamagotchi, TmAction, TmEvent};

pub struct TamagotchiFactoryMetadata;

impl Metadata for TamagotchiFactoryMetadata {
    type Init = In<InitTamagotchiFactory>;
    type Handle = InOut<TMFAction, TMFEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = ();
}

const GAS_FOR_CREATION: u64 = 100_000_000;

type TamagotchiId = u64;

#[derive(Decode, Encode, Default, Debug, TypeInfo)]
pub struct InitTamagotchiFactory {
    pub tamagotchi_template: CodeId,
    pub tamagotchi_bank: ActorId,
}

#[derive(Decode, Encode, Default, Debug, TypeInfo)]
pub struct TamagotchiFactory {
    pub tamagotchi_template: CodeId,
    pub tamagotchi_number: TamagotchiId,
    pub id_to_address: BTreeMap<TamagotchiId, ActorId>,
    pub tamagotchi_bank: ActorId,
}

impl TamagotchiFactory {
    pub async fn create_tamagotchi(&mut self, name: String) {
        debug!("Tamagotchi name: {:?}", name);
        let (address, _) = ProgramGenerator::create_program_with_gas_for_reply(
            self.tamagotchi_template,
            InitTamagotchi {
                commander: exec::program_id(),
                name: name,
                owner: msg::source(),
                tamagotchi_bank: self.tamagotchi_bank,
            }
            .encode(),
            GAS_FOR_CREATION,
            0,
            0,
        )
        .expect("Error creatubg tamagotchi")
        .await
        .expect("Program was not initialized");
        self.tamagotchi_number = self.tamagotchi_number.saturating_add(1);
        debug!("Tamagotchi id: {:?}", self.tamagotchi_number);
        debug!("Address: {:?}", address);
        self.id_to_address.insert(self.tamagotchi_number, address);
        debug!("Tamagotchi succesfully created");
        msg::reply(
            TMFEvent::TamagotchiCreated {
                tamagotchi_id: self.tamagotchi_number,
                tamagotchi_address: address,
            },
            0,
        )
        .expect("Error replying tamagotchi creation");
    }

    pub async fn process_tm_action(
        &self,
        tamagotchi: TamagotchiId,
        tm_action: _TmAction,
        source: &ActorId,
    ) {
        let tamagotchi = self.get_tamagotchi_address(tamagotchi);
        debug!("Tamagotchi address: {:?}", tamagotchi);
        debug!("Tamagotchi action: {:?}", tm_action);
        match tm_action {
            _TmAction::Name => {
                send_message(&tamagotchi, TmAction::Name).await;
            }
            _TmAction::Age => {
                send_message(&tamagotchi, TmAction::Age).await;
            }
            _TmAction::Feed => {
                debug!("Sending feed message");
                send_message(&tamagotchi, TmAction::Feed(*source)).await;
            }
            _TmAction::Sleep => {
                debug!("Sending sleep message");
                send_message(&tamagotchi, TmAction::Sleep(*source)).await;
            }
            _TmAction::Play => {
                debug!("Sending play message");
                send_message(&tamagotchi, TmAction::Play(*source)).await;
            }
            _TmAction::Transfer(to) => {
                send_message(&tamagotchi, TmAction::Transfer { from: *source, to }).await;
            }
            _TmAction::Approve(to) => {
                send_message(&tamagotchi, TmAction::Approve { from: *source, to }).await;
            }
            _TmAction::RevokeApproval => {
                send_message(&tamagotchi, TmAction::RevokeApproval(*source)).await;
            }
            _TmAction::SetTokenContract(new_bank) => {
                send_message(
                    &tamagotchi,
                    TmAction::SetTokenContract {
                        source: *source,
                        new_bank,
                    },
                )
                .await;
            }
            _TmAction::ApproveTokens { account, amount } => {
                send_message(
                    &tamagotchi,
                    TmAction::ApproveTokens {
                        source: *source,
                        account,
                        amount,
                    },
                )
                .await;
            }
            _TmAction::BuyAttribute {
                store_id,
                attribute_id,
            } => {
                send_message(
                    &tamagotchi,
                    TmAction::BuyAttribute {
                        source: *source,
                        store_id,
                        attribute_id,
                    },
                )
                .await;
            }
            _TmAction::Owner => {
                send_message(&tamagotchi, TmAction::Owner).await;
            }
            _TmAction::CheckState => {
                send_message(&tamagotchi, TmAction::CheckState(*source)).await;
            }
            _TmAction::ReserveGas {
                reservation_amount,
                duration,
            } => {
                send_message(
                    &tamagotchi,
                    TmAction::ReserveGas {
                        source: *source,
                        reservation_amount,
                        duration,
                    },
                )
                .await;
            }
        }
    }

    fn get_tamagotchi_address(&self, tamagotchi_id: TamagotchiId) -> ActorId {
        debug!("Tamagotchi id: {:?}", tamagotchi_id);
        *self
            .id_to_address
            .get(&tamagotchi_id)
            .expect("Tamagotchi not found")
    }
}

async fn send_message(tamagotchi_address: &ActorId, action: TmAction) {
    debug!("Sending message to tamagotchi");
    let res = msg::send_for_reply_as::<_, TmEvent>(tamagotchi_address.clone(), action, 0, 0)
        .expect("Error sending message")
        .await
        .expect("Unnable to decode TmEvent");
    debug!("Tamagotchi result {:?}", res);
    msg::reply(res, 0).expect("Cannot send TmEvent reply");
}

#[derive(Decode, Encode, Debug, TypeInfo)]
pub enum TMFAction {
    CreateTamagotchi(String),
    TmAction(TamagotchiId, _TmAction),
}

#[derive(Decode, Encode, Debug, TypeInfo)]
pub enum _TmAction {
    Name,
    Age,
    Feed,
    Sleep,
    Play,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    SetTokenContract(ActorId),
    ApproveTokens {
        account: ActorId,
        amount: u128,
    },
    BuyAttribute {
        store_id: ActorId,
        attribute_id: AttributeId,
    },
    Owner,
    CheckState,
    ReserveGas {
        reservation_amount: u64,
        duration: u32,
    },
}

#[derive(Decode, Encode, Debug, TypeInfo)]
pub enum TMFEvent {
    TamagotchiCreated {
        tamagotchi_id: TamagotchiId,
        tamagotchi_address: ActorId,
    },
    TamagotchiEvent(TamagotchiId, TmEvent),
}

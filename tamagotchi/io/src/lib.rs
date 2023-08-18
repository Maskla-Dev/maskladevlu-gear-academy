#![no_std]
use ft_main_io::{FTokenAction, FTokenEvent, LogicAction};
use gmeta::{In, InOut, Metadata};
use gstd::{debug, msg, prelude::*, ActorId, Debug, Decode, Encode, TypeInfo};
use store_io::{AttributeId, TransactionId};

pub struct TamagotchiMetadata;

impl Metadata for TamagotchiMetadata {
    type Init = In<String>;
    type Handle = InOut<TmAction, TmEvent>;
    type Signal = ();
    type Reply = ();
    type Others = ();
    type State = TamagotchiState;
}

pub const HUNGER_PER_BLOCK: u64 = 1;
pub const ENERGY_PER_BLOCK: u64 = 2;
pub const BOREDOM_PER_BLOCK: u64 = 2;
pub const FILL_PER_SLEEP: u64 = 1000;
pub const FILL_PER_FEED: u64 = 1000;
pub const FILL_PER_ENTERTAINMENT: u64 = 1000;
pub const MAX_MOOD_VALUE: u64 = 10000;
pub const MIN_MOOD_VALUE: u64 = 1;

#[derive(Default, Encode, Decode, TypeInfo, Debug)]
pub struct TamagotchiState {
    pub name: String,
    pub date_of_birth: u64,
    pub owner: ActorId,
    pub fed: u64,
    pub fed_block: u64,
    pub entertained: u64,
    pub entertained_block: u64,
    pub rested: u64,
    pub rested_block: u64,
    pub allowed_account: Option<ActorId>,
    pub ft_contract: Option<ActorId>,
    pub transaction_id: u64,
    pub approve_transaction: Option<(TransactionId, ActorId, u128)>,
}

impl TamagotchiState {
    pub fn feed(&mut self, current_block_height: u64) {
        debug!("Fed block {:?}", self.fed_block);
        debug!("Total Height {:?}", current_block_height - self.fed_block);
        self.fed_block = current_block_height;
        self.fed += FILL_PER_FEED
            .saturating_sub(HUNGER_PER_BLOCK * (current_block_height - self.fed_block));
        TamagotchiState::verify_limit(&mut self.fed);
    }

    pub fn play(&mut self, current_block_height: u64) {
        self.entertained_block = current_block_height;
        self.entertained += FILL_PER_ENTERTAINMENT
            .saturating_sub(BOREDOM_PER_BLOCK * (current_block_height - self.entertained_block));
        TamagotchiState::verify_limit(&mut self.entertained);
    }

    pub fn sleep(&mut self, current_block_height: u64) {
        TamagotchiState::verify_limit(&mut self.rested);
        self.rested_block = current_block_height;
        self.rested += FILL_PER_SLEEP
            .saturating_sub(ENERGY_PER_BLOCK * (current_block_height - self.rested_block));
        TamagotchiState::verify_limit(&mut self.rested);
    }

    pub fn verify_limit(mood_param: &mut u64) {
        if *mood_param > MAX_MOOD_VALUE {
            *mood_param = MAX_MOOD_VALUE;
        }
        if *mood_param < MIN_MOOD_VALUE {
            *mood_param = MIN_MOOD_VALUE;
        }
    }

    pub fn verify_ownership(&self, source: ActorId) -> bool {
        self.owner == source
    }

    pub fn verify_allowed_account(&self, source: ActorId) -> bool {
        self.allowed_account == Some(source)
    }

    pub fn verify_permission(&self, source: ActorId) -> bool {
        self.verify_ownership(source) || self.verify_allowed_account(source)
    }

    pub async fn approve_tokens(&mut self, account: &ActorId, amount: u128) -> TmEvent {
        let (transaction_id, amount) = if let Some((
            ft_transaction_id,
            prev_account,
            prev_amount,
        )) = self.approve_transaction
        {
            if prev_account != *account || prev_amount != amount {
                panic!("Please complete the previous transaction");
            } else {
                (ft_transaction_id, prev_account, prev_amount)
            }
        } else {
            let ft_transaction_id = self.transaction_id;
            self.transaction_id = self.transaction_id.wrapping_add(1);
            self.approve_transaction = Some((ft_transaction_id, *account, amount));
            (ft_transaction_id, *account, amount)
        };
        if let Some(contract) = self.ft_contract {
            debug!("Sending approve tokens message to FT contract");
            let result = msg::send_for_reply_as::<_, FTokenEvent>(
                contract,
                FTokenAction::Message {
                    transaction_id,
                    payload: LogicAction::Approve {
                        approved_account: account,
                        amount,
                    },
                },
                0,
                0,
            )
            .expect("Error sending approve tokens message")
            .await;
            self.approve_transaction = None;
            match result {
                Ok(_) => return TmEvent::Approve(account),
                Err(_) => return TmEvent::ApprovalError,
            }
        } else {
            debug!("FT contract not set");
            panic!("FT contract not set");
        };
    }
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmAction {
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
    Owner
}

#[derive(Encode, Decode, TypeInfo, Debug)]
pub enum TmEvent {
    Name(String),
    Age(u64),
    Fed,
    Entertained,
    Slept,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
    TokenContractSet,
    TokensApproved { account: ActorId, amount: u128 },
    ApprovalError,
    AttributeBought(AttributeId),
    CompletePrevPurchase(AttributeId),
    ErrorDuringPurchase,
    Owner(ActorId)
}

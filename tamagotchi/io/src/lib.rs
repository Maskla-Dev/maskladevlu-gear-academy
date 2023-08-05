#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{debug, prelude::*, ActorId, Debug, Decode, Encode, TypeInfo};

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
}

impl TamagotchiState {
    pub fn feed(&mut self, current_block_height: u64) {
        debug!("Fed block {:?}", self.fed_block);
        debug!("Total Height {:?}", current_block_height - self.fed_block);
        self.fed_block = current_block_height;
        self.fed += FILL_PER_FEED - (HUNGER_PER_BLOCK * (current_block_height - self.fed_block));
        TamagotchiState::verify_limit(&mut self.fed);
    }

    pub fn play(&mut self, current_block_height: u64) {
        self.entertained_block = current_block_height;
        self.entertained += FILL_PER_ENTERTAINMENT
            - (BOREDOM_PER_BLOCK * (current_block_height - self.entertained_block));
        TamagotchiState::verify_limit(&mut self.entertained);
    }

    pub fn sleep(&mut self, current_block_height: u64) {
        TamagotchiState::verify_limit(&mut self.rested);
        self.rested_block = current_block_height;
        self.rested +=
            FILL_PER_SLEEP - (ENERGY_PER_BLOCK * (current_block_height - self.rested_block));
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
        match self.allowed_account {
            Some(account) => account == source,
            None => false,
        }
    }
    pub fn verify_permission(&self, source: ActorId) -> bool {
        self.verify_ownership(source) || self.verify_allowed_account(source)
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
}

#[derive(Encode, Decode, TypeInfo)]
pub enum TmEvent {
    Name(String),
    Age(u64),
    Fed,
    Entertained,
    Slept,
    Transfer(ActorId),
    Approve(ActorId),
    RevokeApproval,
}

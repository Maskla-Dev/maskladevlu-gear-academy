#![no_std]
use gmeta::Metadata;
use gstd::{msg, prelude::*, ActorId, Debug, Decode, Encode, ReservationId, TypeInfo};
use tamagotchi_io::*;

pub struct AuctionMetadata;

impl Metadata for AuctionMetadata {
    type Init = ();
    type State = ();
    type Handle = ();
    type Reply = ();
    type Others = ();
    type Signal = ();
}

#[derive(Encode, Decode, TypeInfo, Debug, Default)]
pub struct Auction {
    pub auctioner: ActorId,
    tamagotchi_id: ActorId,
    status: Status,
    current_bid: u128,
    current_bidder: ActorId,
    ft_contract_id: ActorId,
    transaction_id: TransactionId,
    ended_at: u64,
    prev_tmg_owner: ActorId,
    pub transaction: Option<Transaction>,
    pub reservations: Vec<ReservationId>,
}

const MIN_DURATION: u64 = 60;
const RESERVATION_AMOUNT: u64 = 50_000_000_000;
const RESERVATION_DURATION: u32 = 86400;

impl Auction {
    pub async fn start_auction(
        &mut self,
        tamagotchi_id: &TamagotchiId,
        minimum_bid: Bid,
        duration: u64,
        current_time: u64,
        source: ActorId,
    ) -> Result<AuctionEvent, AuctionError> {
        if self.status != Status::ReadyToStart {
            return Err(AuctionError::WrongState);
        }
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::StartAuction {
                    tamagotchi_id: prev_tmg_id,
                    bid,
                    duration: prev_duration,
                } => {
                    if *tamagotchi_id != prev_tmg_id
                        || bid != minimum_bid
                        || duration != prev_duration
                    {
                        return Err(AuctionError::WrongParams);
                    }
                    return self
                        .complete_tx(
                            Transaction::StartAuction {
                                tamagotchi_id: *tamagotchi_id,
                                bid: bid,
                                duration: duration,
                            },
                            &source,
                            current_time,
                        )
                        .await;
                }
                _ => return Err(AuctionError::WrongTx),
            }
        }
        if duration < MIN_DURATION {
            return Err(AuctionError::WrongDuration);
        }
        let tx = Transaction::StartAuction {
            tamagotchi_id: *tamagotchi_id,
            bid: minimum_bid,
            duration: duration,
        };
        self.transaction = Some(tx.clone());
        self.complete_tx(tx, &source, current_time).await
    }

    pub async fn complete_tx(
        &mut self,
        tx: Transaction,
        source: &ActorId,
        current_time: u64,
    ) -> Result<AuctionEvent, AuctionError> {
        match tx {
            Transaction::StartAuction {
                tamagotchi_id,
                bid,
                duration,
            } => {
                let tmg_owner = if let Ok(tmg_owner) = get_owner(&self.tamagotchi_id).await {
                    tmg_owner
                } else {
                    return Err(AuctionError::WrongReceivedMessage);
                };
                if tmg_owner == self.auctioner {
                    self.status = Status::InProcess;
                    self.current_bid = bid;
                    self.transaction = None;
                    self.ended_at = current_time + duration;
                    return Ok(AuctionEvent::AuctionStarted);
                }
                if tmg_owner != *source {
                    return Err(AuctionError::NotOwner);
                }
                if change_owner(&tamagotchi_id, &self.auctioner).await.is_err() {
                    self.transaction = None;
                    return Err(AuctionError::UnableToChangeOwner);
                } else {
                    self.status = Status::InProcess;
                    self.current_bid = bid;
                    self.prev_tmg_owner = tmg_owner;
                    self.ended_at = current_time + duration;
                    self.transaction = None;
                    msg::send_delayed(self.auctioner, AuctionAction::SettleAuction, 0, 0)
                        .expect("Error sending delayed message at complete_tx");
                    Ok(AuctionEvent::AuctionStarted)
                }
            }
            Transaction::MakeBid {
                transaction_id,
                bidder,
                bid,
            } => {
                if transfer_tokens(
                    transaction_id,
                    &self.ft_contract_id,
                    &bidder,
                    &self.auctioner,
                    bid,
                )
                .await
                .is_err()
                {
                    self.transaction = None;
                    return Err(AuctionError::UnableToTransferTokens);
                }

                // If it's not the first bid,
                // we have to return the tokens to the previous bidder
                // since the tokens are on the auction contract
                // The transaction can fail only due to a lack of gas
                // It's necessary to rerun the transaction
                if !self.current_bidder.is_zero()
                    && transfer_tokens(
                        transaction_id + 1,
                        &self.ft_contract_id,
                        &self.auctioner,
                        &self.current_bidder,
                        self.current_bid,
                    )
                    .await
                    .is_err()
                {
                    return Err(AuctionError::RerunTransaction);
                }

                self.current_bid = bid;
                self.current_bidder = bidder;
                Ok(AuctionEvent::BidMade { bid })
            }
            Transaction::SettleAuction { transaction_id } => {
                let tmg_owner = if let Ok(tmg_owner) = get_owner(&self.tamagotchi_id).await {
                    tmg_owner
                } else {
                    return Err(AuctionError::WrongReceivedMessage);
                };
                if tmg_owner == self.auctioner {
                    if self.current_bidder.is_zero() {
                        if change_owner(&self.tamagotchi_id, &self.prev_tmg_owner)
                            .await
                            .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };
                    } else {
                        if transfer_tokens(
                            transaction_id,
                            &self.ft_contract_id,
                            &self.auctioner,
                            &self.prev_tmg_owner,
                            self.current_bid,
                        )
                        .await
                        .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };

                        if change_owner(&self.tamagotchi_id, &self.current_bidder)
                            .await
                            .is_err()
                        {
                            return Err(AuctionError::RerunTransaction);
                        };
                    }
                }
                self.transaction = None;
                self.prev_tmg_owner = ActorId::zero();
                self.current_bidder = ActorId::zero();
                self.status = Status::ReadyToStart;
                self.ended_at = 0;
                self.tamagotchi_id = ActorId::zero();

                Ok(AuctionEvent::AuctionSettled)
            }
            _ => {
                return Err(AuctionError::WrongTx);
            }
        }
    }

    pub async fn make_bid(
        &mut self,
        bid: Bid,
        source: &ActorId,
        current_time: u64,
    ) -> Result<AuctionEvent, AuctionError> {
        if self.status != Status::InProcess {
            return Err(AuctionError::WrongState);
        }
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::MakeBid {
                    transaction_id,
                    bidder,
                    bid: prev_bid,
                } => {
                    let result = self.complete_tx(tx, source, current_time).await;
                    if bidder == *source && bid == prev_bid {
                        return result;
                    }
                }
                _ => {
                    return Err(AuctionError::WrongTx);
                }
            }
        }
        if bid <= self.current_bid {
            return Err(AuctionError::WrongBid);
        }
        let transaction_id = self.transaction_id;
        let bidder = source;
        self.transaction_id = self.transaction_id.wrapping_add(2);
        let tx = Transaction::MakeBid {
            transaction_id,
            bidder: *bidder,
            bid,
        };
        self.transaction = Some(tx.clone());
        self.complete_tx(tx, source, current_time).await
    }

    pub async fn settle_auction(
        &mut self,
        source: &ActorId,
        current_time: u64,
    ) -> Result<AuctionEvent, AuctionError> {
        if self.ended_at < current_time {
            return Err(AuctionError::WrongState);
        }
        if let Some(tx) = self.transaction.clone() {
            match tx {
                Transaction::MakeBid { .. } => {
                    return self.complete_tx(tx, source, current_time).await
                }
                Transaction::SettleAuction { .. } => {
                    return self.complete_tx(tx, source, current_time).await
                }
                _ => {
                    return Err(AuctionError::WrongTx);
                }
            }
        }
        let transaction_id = self.transaction_id;
        self.transaction_id = self.transaction_id.wrapping_add(1);
        let tx = Transaction::SettleAuction {
            transaction_id: transaction_id,
        };
        self.transaction = Some(tx.clone());
        return self.complete_tx(tx, source, current_time).await;
    }

    pub fn make_reservation(&mut self) -> Result<AuctionEvent, AuctionError> {
        let reservation_id = ReservationId::reserve(RESERVATION_AMOUNT, RESERVATION_DURATION)
            .expect("Reservation across executions");
        self.reservations.push(reservation_id);
        Ok(AuctionEvent::ReservationMade)
    }
}

pub async fn get_owner(tamagotchi_id: &TamagotchiId) -> Result<ActorId, AuctionError> {
    let reply = msg::send_for_reply_as(*tamagotchi_id, TmAction::Owner, 0, 0)
        .expect("Error getting owner")
        .await;
    match reply {
        Ok(TmEvent::Owner(tmg_owner)) => Ok(tmg_owner),
        _ => Err(AuctionError::WrongReceivedMessage),
    }
}

pub async fn change_owner(
    tamagotchi_id: &TamagotchiId,
    new_owner: &ActorId,
) -> Result<TmEvent, ContractError> {
    msg::send_for_reply_as::<_, TmEvent>(*tamagotchi_id, TmAction::Transfer(*new_owner), 0, 0)
        .expect("Error during change owner")
        .await
}

async fn transfer_tokens(
    transaction_id: TransactionId,
    ft_contract_id: &ActorId,
    bidder: &ActorId,
    auctioner: &ActorId,
    bid: u128,
) -> Result<FTokenEvent, AuctionError> {
}

#[derive(Debug, Default, PartialEq, Eq, Encode, Decode, TypeInfo)]
enum Status {
    #[default]
    ReadyToStart,
    InProcess,
}

pub type TamagotchiId = ActorId;
pub type Bid = u128;
pub type TransactionId = u64;

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum AuctionAction {
    StartAuction {
        tamagotchi_id: TamagotchiId,
        minimum_bid: Bid,
        duration: u64,
    },
    MakeBid {
        bid: Bid,
    },
    SettleAuction,
    MakeReservation,
    CompleteTx(Transaction),
}

#[derive(Clone, Decode, Encode, TypeInfo, Debug)]
pub enum Transaction {
    StartAuction {
        tamagotchi_id: ActorId,
        bid: Bid,
        duration: u64,
    },
    MakeBid {
        transaction_id: TransactionId,
        bidder: ActorId,
        bid: u128,
    },
    SettleAuction {
        transaction_id: TransactionId,
    },
    MakeReservation,
}

#[derive(Decode, Encode, TypeInfo, Debug)]
enum AuctionEvent {
    AuctionStarted,
    BidMade { bid: Bid },
    AuctionSettled,
    ReservationMade,
}

#[derive(Decode, Encode, TypeInfo, Debug)]
enum AuctionError {
    WrongState,
    WrongParams,
    WrongReceivedMessage,
    NotOwner,
    UnableToChangeOwner,
    WrongTx,
    WrongDuration,
    WrongBid,
    RerunTransaction,
    UnableToTransferTokens,
}

pub enum ContractError {}

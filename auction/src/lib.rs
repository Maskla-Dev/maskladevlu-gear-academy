#![no_std]
use auction_io::{Auction, AuctionAction};
use gstd::{exec, msg, prelude::*};

static mut AUCTION: Option<Auction> = None;

#[async_main]
extern "C" fn main() {
    let action: AuctionAction = msg::load().expect("Unable to decode AuctionAction");
    let auction = unsafe { AUCTION.get_or_insert(Default::default()) };
    let reply = match action {
        AuctionAction::StartAuction {
            tamagotchi_id,
            minimum_bid,
            duration,
        } => {
            system_reserve_gas();
            auction
                .start_auction(
                    &tamagotchi_id,
                    minimum_bid,
                    duration,
                    exec::block_timestamp(),
                    msg::source(),
                )
                .await;
        }
        AuctionAction::MakeBid { bid } => {
            system_reserve_gas();
            auction
                .make_bid(bid, &msg::source(), exec::block_timestamp())
                .await;
        }
        AuctionAction::SettleAuction => {
            system_reserve_gas();
            auction
                .settle_auction(&msg::source(), exec::block_timestamp())
                .await;
        }
        Auction::MakeReservation => {
            auction.make_reservation();
        }
        AuctionAction::CompleteTx(Tx) => {
            let result = if let Some(_tx) = &auction.transaction {
                if tx == _tx.clone() {
                    auction
                        .complete_tx(tx, &msg::source(), exec::block_timestamp())
                        .await;
                } else {
                    Err(AuctionErrot::WrongTx);
                }
            } else {
                Err(AuctionError::NoTx)
            };
            result
        }
    };
    msg::reply(reply, 0).expect("Failed to encode or reply with `Result<MarketEvent, MarketErr>`");
}

fn system_reserve_gas() {
    exec::system_reserve_gas(SYSTEM_GAS).expect("Error during gas reservation");
}

#[no_mangle]
extern "C" fn my_handle_signal() {
    let auction = unsafe { AUCTION.get_or_insert(Default::default()) };
    if let Some(tx) = &auction.transaction {
        let reservation_id = if !auction.reservations.is_empty() {
            auction.reservations.remove(0)
        } else {
            return;
        };
        msg::send_from_reservation(
            reservation_id,
            auction.auctioner,
            AuctionAction::CompleteTx(tx.clone),
            0,
        )
        .expect("Failed sending from handle_signal");
    }
}

#[no_mangle]
extern "C" fn init() {}

// #[no_mangle]
// extern "C" fn state{

// }

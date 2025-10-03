use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum Side {
    BUY, SELL
}

type ProgramSource = String;

enum Value {
    Word([u64; 4]),
    Imm(u64),
}

type Param = (String, Value);

type ParamList = HashMap<String, Value>;

enum NoteType {
    Public,
    Private,
}

struct AbstactNote {
    schema: String,

    inputs: ParamList,
    program_source: ProgramSource,
}

type Market = String;
type UUID = String;
type Amount = u64;
type Price = u64;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "data")]
enum Note {
    // Notes emited by Desk, consumed by Client
    KYCPassed {
        market: Market,
    },
    QuoteRequestOffer {
        market: Market,
        uuid: UUID,
        side: Side,
        amount: Amount,
        price: Price,
    },
    QuoteRequestNoOffer {
        market: Market,
        uuid: UUID,
    },
    LimitBuyOrderLocked,
    LimitBuyOrderNotLocked, // At that stage the order is firm
    LimitSellOrderLocked,
    LimitSellOrderNotLocked,

    // Notes emitted by Client, consumed by Desk
    QuoteRequest {
        market: Market,
        uuid: UUID,
        side: Side,
        amount: Amount,
    },
    LimitBuyOrder,
    LimitSellOrder,
}

struct MidenNote {
    schema: String,
    note_type: Note,
    inputs: ParamList,
    program: ProgramSource,
}

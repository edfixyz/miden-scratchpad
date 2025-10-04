use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum Side {
    BUY, SELL
}

type ProgramSource = String;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
enum Value {
    Word([u64; 4]),
    Imm(u64),
}

type Input = (String, Value);

#[derive(PartialEq, Serialize, Deserialize, Debug)]
struct Inputs(HashMap<String, Value>);

#[derive(PartialEq, Serialize, Deserialize, Debug)]
enum NoteType {
    Public,
    Private,
}

struct AbstactNote {
    schema: String,

    inputs: Inputs,
    program_source: ProgramSource,
}

type Market = String;
type UUID = String;
type Amount = u64;
type Price = u64;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
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
    LimitOrder {
        market: Market,
        uuid: UUID,
        side: Side,
        amount: Amount,
        price: Price,
    },
}

#[derive(PartialEq, Serialize, Deserialize, Debug)]
struct MidenAbstractNote {
    schema: String,
    note: Note,
    note_type: NoteType,
    program: ProgramSource,
    libraries: Vec<ProgramSource>,
}

type Recipient = String;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
struct MidenNote {
    schema: String,
    note_type: NoteType,
    recipient: Recipient,
    miden_note_hex: String
}

// fn abstract_to_miden(abstract_note: MidenAbstractNote) -> MidenNote {
    
// }







pub fn play() {
    let note = MidenAbstractNote {
        schema: "EDFI 0 MIDEN 0.18".to_string(),
        note: Note::LimitBuyOrderLocked,
        note_type: NoteType::Private,
        program: "/abc.masm".to_string(),
        libraries: vec!["/lib.masm".to_string(), "/lib2.masm".to_string()],
    };

    let note_json = serde_json::to_string(&note).unwrap();
    println!("{}", note_json);
}
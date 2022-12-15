#![feature(buf_read_has_data_left)]
use anyhow::Result;
use bytes::Buf;
use clap::Parser;
use cli::Args;
use regex::Regex;
use rlp::Rlp;
use serde::Serialize;
use std::io::{self, Read};
use thiserror::Error;

mod cli;
mod convert;
mod rlp;
mod transaction;

const BASE_64_REGEX: &str = "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";
const HEX_REGEX: &str = "^(0x)?([0-9A-Fa-f]{2})+$";

#[derive(Debug, Error)]
enum Error {
    #[error("Invalid transaction format")]
    FormatError,
    #[error("Invalid transaction RLP")]
    RlpError,
}

fn main() {
    let args = Args::parse();
    let transaction = args.transaction;

    if transaction.is_empty() {
        read_stdin_tx(args.estimate);
        return;
    }

    handle_transaction(&transaction, args.estimate);
}

fn read_stdin_tx(estimate: bool) {
    let mut buffer = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut buffer) {
        println!("Unable to read transaction: {error:?}");
        return;
    }

    handle_transaction(&buffer, estimate);
}

fn handle_transaction(transaction: &str, estimate: bool) {
    match decode_transaction(transaction, estimate) {
        Ok(TransactionData::SubmittableTransaction(transaction)) => println!(
            "{}",
            serde_json::to_string_pretty(&transaction).expect("JSON should serialize")
        ),
        Ok(TransactionData::TransactionWithSender(address, transaction)) => println!(
            "Sender: {address},\ntransaction: {}",
            serde_json::to_string_pretty(&transaction).expect("JSON should serialize")
        ),
        Err(error) => println!("Unable to decode transaction: {error:?}"),
    }
}

fn decode_transaction(transaction: &str, estimate: bool) -> Result<TransactionData> {
    let transaction = transaction.trim();
    let hex_regex = Regex::new(HEX_REGEX).expect("Regex should compile");
    if hex_regex.is_match(transaction) {
        let transaction = if let Some(transaction) = transaction.strip_prefix("0x") {
            transaction
        } else {
            transaction
        };
        let raw_transaction = hex::decode(transaction)?;
        return parse_eth_tx(&raw_transaction, estimate);
    }
    let base64_regex = Regex::new(BASE_64_REGEX).expect("Regex should compile");
    if base64_regex.is_match(transaction) {
        let raw_transaction = base64::decode(transaction)?;
        return parse_eth_tx(&raw_transaction, estimate);
    }

    Err(Error::FormatError.into())
}

#[derive(Debug, Serialize)]
struct Transaction {
    nonce: u128,
    gas: u128,
    gas_limit: u128,
    target_address: String,
    value: u128,
    data: String,
    v: String,
    r: String,
    s: String,
}

enum TransactionData {
    TransactionWithSender(String, Transaction),
    SubmittableTransaction(Transaction),
}

fn parse_eth_tx(data: &[u8], estimate: bool) -> Result<TransactionData> {
    let mut reader = data.reader();
    let mut address = None;
    if estimate {
        let mut address_bytes = vec![0u8; 20];
        reader.read_exact(&mut address_bytes)?;
        address = Some(format!("0x{}", hex::encode(address_bytes)));
    }

    let tx_node = rlp::parse_element(&mut reader)?;

    let Rlp::List(list) = tx_node else {
        return Err(Error::RlpError.into());
    };

    if list.len() < 8 {
        return Err(Error::RlpError.into());
    }

    let transaction = Transaction {
        nonce: convert::to_big_int(&list[0])?,
        gas: convert::to_big_int(&list[1])?,
        gas_limit: convert::to_big_int(&list[2])?,
        target_address: convert::to_string(&list[3])?,
        value: convert::to_big_int(&list[4])?,
        data: convert::to_string(&list[5])?,
        v: convert::to_string(&list[6])?,
        r: convert::to_string(&list[7])?,
        s: convert::to_string(&list[8])?,
    };

    match address {
        Some(address) => Ok(TransactionData::TransactionWithSender(address, transaction)),
        None => Ok(TransactionData::SubmittableTransaction(transaction)),
    }
}

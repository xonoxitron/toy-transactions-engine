use crate::types::{Account, Transaction};
use csv::{ReaderBuilder, Trim};
use std::error::Error;

pub fn process_csv(path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(&path)?;
    let mut transactions: Vec<Transaction> = Vec::new();
    for result in reader.deserialize() {
        let record: Transaction = result?;
        transactions.push(record);
    }
    Ok(transactions)
}

pub fn process_output(processed_txs: Vec<Account>, tx_errs: Vec<String>, output_tx_errs: bool) {
    if output_tx_errs {
        for err in tx_errs {
            println!("{}", err)
        }
    }
    println!("client,available,held,total,locked");
    for tx in processed_txs {
        println!(
            "{},{},{},{},{}",
            tx.client, tx.available, tx.held, tx.total, tx.locked
        )
    }
}

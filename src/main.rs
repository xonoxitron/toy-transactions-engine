mod engine;
mod helpers;
mod types;

use tokio;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        let args: Vec<String> = std::env::args().collect();
        let args_len = args.len();
        if args_len > 1 && args[1].ends_with(".csv") {
            match helpers::process_csv(&args[1]) {
                Ok(txs) => {
                    let (processed_txs, tx_errs) = engine::process_transactions(txs);
                    let mut output_tx_errs = false;
                    if args_len > 2 {
                        output_tx_errs = args[2] == "true" || args[2] == "1";
                    }
                    helpers::process_output(processed_txs, tx_errs, output_tx_errs);
                }
                Err(err) => {
                    println!("error parsing csv: {}", err);
                }
            }
        } else {
            println!("*.csv input file not found");
        }
    });
}

use crate::types::{Account, Transaction};
use rust_decimal::Decimal;
use std::collections::HashMap;

pub fn process_transactions(transactions: Vec<Transaction>) -> (Vec<Account>, Vec<String>) {
    let mut accounts: HashMap<u16, Account> = HashMap::new();
    let mut applied_txs: HashMap<u32, Decimal> = HashMap::new();
    let mut disputed_txs: HashMap<u32, Decimal> = HashMap::new();
    let mut tx_errors: Vec<String> = Vec::new();

    for transaction in transactions {
        let account = accounts
            .entry(transaction.client)
            .or_insert_with(|| Account::empty(transaction.client));

        match transaction.transaction_type.as_str() {
            "deposit" => {
                account.deposit(transaction.amount).unwrap();
                applied_txs.insert(transaction.tx, transaction.amount);
            }
            "withdrawal" => {
                match account.withdraw(transaction.amount) {
                    Ok(_) => applied_txs.insert(transaction.tx, transaction.amount),
                    Err(err) => {
                        tx_errors.push(format!(
                            "Error when handling transaction \"{}\": {}",
                            transaction.tx, err
                        ));
                        continue;
                    }
                };
            }
            "dispute" => {
                let disputable = match applied_txs.get(&transaction.tx) {
                    Some(disputable) => disputable.clone(),
                    None => {
                        tx_errors.push(format!(
                            "Could not find applied transaction \"{}\" to dispute",
                            transaction.tx
                        ));
                        continue;
                    }
                };

                match disputed_txs.get(&transaction.tx) {
                    Some(_) => {
                        tx_errors.push(format!(
                            "Could not dispute same transaction \"{}\" twice",
                            transaction.tx
                        ));
                        continue;
                    }
                    None => {}
                };

                match account.dispute(disputable) {
                    Ok(_) => disputed_txs.insert(transaction.tx, disputable),
                    Err(err) => {
                        tx_errors.push(format!(
                            "Could not dispute transaction \"{}\": {}",
                            transaction.tx, err
                        ));
                        continue;
                    }
                };
            }
            "resolve" => {
                let resolvable = match disputed_txs.get(&transaction.tx) {
                    Some(amount) => amount.clone(),
                    None => {
                        tx_errors.push(format!(
                            "Could not find disputed transaction \"{}\" to resolve",
                            transaction.tx
                        ));
                        continue;
                    }
                };

                match account.resolve(resolvable) {
                    Ok(_) => disputed_txs.remove(&transaction.tx),
                    Err(err) => {
                        tx_errors.push(format!(
                            "Could not resolve disputed transaction \"{}\": {}",
                            transaction.tx, err
                        ));
                        continue;
                    }
                };
            }
            "chargeback" => {
                let back_chargeable = match disputed_txs.get(&transaction.tx) {
                    Some(amount) => amount.clone(),
                    None => {
                        tx_errors.push(format!(
                            "Could not find disputed transaction \"{}\" to charge back",
                            transaction.tx
                        ));
                        continue;
                    }
                };

                match account.chargeback(back_chargeable) {
                    Ok(_) => disputed_txs.remove(&transaction.tx),
                    Err(err) => {
                        tx_errors.push(format!(
                            "Could not charge back disputed transaction \"{}\": {}",
                            transaction.tx, err
                        ));
                        continue;
                    }
                };
            }
            t => {
                tx_errors.push(format!("Unhandled transaction type: \"{}\"", t));
            }
        };
    }

    (
        accounts.into_iter().map(|(_id, account)| account).collect(),
        tx_errors,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{Account, Transaction};
    use hamcrest::*;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    
    const TEST_CLIENT_ID: u16 = 42;

    #[test]
    fn test_no_transactions() {
        let (accounts, errors) = process_transactions(vec![]);
        assert_that!(accounts, is(equal_to(vec![])));
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_deposit() {
        let (accounts, errors) = process_transactions(vec![Transaction::new(
            "deposit".into(),
            TEST_CLIENT_ID,
            2,
            dec!(3.1234),
        )]);

        assert_that!(
            accounts,
            is(equal_to(vec![Account::new(
                TEST_CLIENT_ID,
                dec!(3.1234),
                dec!(0.0),
                false
            ),]))
        );
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_withdrawal() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 2, dec!(3.1234)),
            Transaction::new("withdrawal".into(), TEST_CLIENT_ID, 2, dec!(3.1234)),
        ]);

        assert_that!(
            accounts,
            is(equal_to(vec![Account::new(
                TEST_CLIENT_ID,
                dec!(0.0),
                dec!(0.0),
                false
            ),]))
        );
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_withdrawal_from_insufficient_funds() {
        let (accounts, errors) = process_transactions(vec![Transaction::new(
            "withdrawal".into(),
            TEST_CLIENT_ID,
            2,
            dec!(3.1234),
        )]);

        assert_that!(
            accounts,
            is(equal_to(vec![Account::new(
                TEST_CLIENT_ID,
                dec!(0.0),
                dec!(0.0),
                false
            )]))
        );
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_dispute() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(0.0), dec!(100.0), dec!(100.0), false);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_cannot_dispute_twice() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 2, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(100.0), dec!(200.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_dispute_for_unknown_transaction() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_resolve() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
            Transaction::new("resolve".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_cannot_resolve_twice() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 2, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
            Transaction::new("resolve".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
            Transaction::new("resolve".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(200.0), dec!(0.0), dec!(200.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_resolve_for_unknown_transaction() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("resolve".into(), TEST_CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_undisputed_resolve() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("resolve".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_chargeback() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
            Transaction::new("chargeback".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(0.0), dec!(0.0), dec!(0.0), true);
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn test_cannot_chargeback_twice() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 2, dec!(100.0)),
            Transaction::new("dispute".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
            Transaction::new("chargeback".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
            Transaction::new("chargeback".into(), TEST_CLIENT_ID, 2, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), true);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_chargeback_for_unknown_transaction() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("chargeback".into(), TEST_CLIENT_ID, 999, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_ignore_undisputed_chargeback() {
        let (accounts, errors) = process_transactions(vec![
            Transaction::new("deposit".into(), TEST_CLIENT_ID, 1, dec!(100.0)),
            Transaction::new("chargeback".into(), TEST_CLIENT_ID, 1, dec!(0.0)),
        ]);

        assert_account(&accounts[0], dec!(100.0), dec!(0.0), dec!(100.0), false);
        assert_eq!(errors.len(), 1);
    }

    fn assert_account(
        account: &Account,
        available: Decimal,
        held: Decimal,
        total: Decimal,
        locked: bool,
    ) {
        assert_that!(account.available, is(equal_to(available)));
        assert_that!(account.held, is(equal_to(held)));
        assert_that!(account.total, is(equal_to(total)));
        assert_that!(account.locked, is(locked));
    }
}

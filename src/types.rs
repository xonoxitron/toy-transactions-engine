use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16, available: Decimal, held: Decimal, locked: bool) -> Self {
        Self {
            client,
            available,
            held,
            total: available + held,
            locked,
        }
    }

    pub fn empty(client: u16) -> Self {
        Self::new(client, Decimal::from(0), Decimal::from(0), false)
    }

    pub fn deposit(&mut self, amount: Decimal) -> Result<(), String> {
        self.available += amount;
        self.total += amount;

        Ok(())
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<(), String> {
        if amount > self.available {
            return Err(format!("Insufficient available funds"));
        }
        self.available -= amount;
        self.total -= amount;

        Ok(())
    }

    pub fn dispute(&mut self, amount: Decimal) -> Result<(), String> {
        if amount > self.available {
            return Err(format!("Insufficient available funds"));
        }
        self.available -= amount;
        self.held += amount;

        Ok(())
    }

    pub fn resolve(&mut self, amount: Decimal) -> Result<(), String> {
        if amount > self.held {
            return Err(format!("Insufficient held funds"));
        }
        self.available += amount;
        self.held -= amount;

        Ok(())
    }

    pub fn chargeback(&mut self, amount: Decimal) -> Result<(), String> {
        if amount > self.held {
            return Err(format!("Insufficient held funds"));
        }
        self.held -= amount;
        self.total -= amount;
        self.locked = true;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename(deserialize = "type"))]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Decimal,
}

#[cfg(test)]
impl Transaction {
    pub fn new(transaction_type: String, client: u16, tx: u32, amount: Decimal) -> Self {
        Self {
            transaction_type,
            client,
            tx,
            amount,
        }
    }
}

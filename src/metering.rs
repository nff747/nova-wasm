//! Instruction metering and execution budget management.

use crate::trap::Trap;

#[derive(Debug, Clone)]
pub struct GasMeter {
    remaining_gas: u64,
    gas_limit: u64,
}

impl GasMeter {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            remaining_gas: gas_limit,
            gas_limit,
        }
    }

    pub fn consume(&mut self, amount: u64) -> Result<(), Trap> {
        if self.remaining_gas < amount {
            self.remaining_gas = 0;
            return Err(Trap::OutOfGas);
        }
        self.remaining_gas -= amount;
        Ok(())
    }

    pub fn remaining(&self) -> u64 {
        self.remaining_gas
    }

    pub fn reset(&mut self) {
        self.remaining_gas = self.gas_limit;
    }
}

// Copyright (c) 2022 MASSA LABS <info@massa.net>

use massa_final_state::StateChanges;
use massa_models::{address::Address, output_event::SCOutputEvent, slot::Slot};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt::Display};

/// The result of the read-only execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate)  enum ReadOnlyResult {
    /// An error occurred during execution.
    Error(String),
    /// The result of a successful execution.
    Ok(Vec<u8>),
}

/// The response to a request for a read-only execution.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate)  struct ExecuteReadOnlyResponse {
    /// The slot at which the read-only execution occurred.
    pub(crate)  executed_at: Slot,
    /// The result of the read-only execution.
    pub(crate)  result: ReadOnlyResult,
    /// The output events generated by the read-only execution.
    pub(crate)  output_events: VecDeque<SCOutputEvent>,
    /// The gas cost for the execution
    pub(crate)  gas_cost: u64,
    /// state changes caused by the execution step
    pub(crate)  state_changes: StateChanges,
}

impl Display for ExecuteReadOnlyResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Executed at slot: {}", self.executed_at)?;
        writeln!(
            f,
            "Result: {}",
            match &self.result {
                ReadOnlyResult::Error(e) =>
                    format!("an error occurred during the execution: {}", e),
                ReadOnlyResult::Ok(ret) => format!("success, returned value: {:?}", ret),
            }
        )?;
        writeln!(f, "Gas cost: {}", self.gas_cost)?;
        if !self.output_events.is_empty() {
            writeln!(f, "Generated events:",)?;
            for event in self.output_events.iter() {
                writeln!(f, "{}", event)?; // id already displayed in event
            }
        }
        Ok(())
    }
}

/// read only bytecode execution request
#[derive(Debug, Deserialize, Clone, Serialize)]
pub(crate)  struct ReadOnlyBytecodeExecution {
    /// max available gas
    pub(crate)  max_gas: u64,
    /// byte code
    pub(crate)  bytecode: Vec<u8>,
    /// caller's address, optional
    pub(crate)  address: Option<Address>,
    /// Operation datastore, optional
    pub(crate)  operation_datastore: Option<Vec<u8>>,
    /// whether to start execution from final or active state. Default false
    #[serde(default)]
    pub(crate)  is_final: bool,
}

/// read SC call request
#[derive(Debug, Deserialize, Clone, Serialize)]
pub(crate)  struct ReadOnlyCall {
    /// max available gas
    pub(crate)  max_gas: u64,
    /// target address
    pub(crate)  target_address: Address,
    /// target function
    pub(crate)  target_function: String,
    /// function parameter
    pub(crate)  parameter: Vec<u8>,
    /// caller's address, optional
    pub(crate)  caller_address: Option<Address>,
    /// whether to start execution from final or active state. Default false
    #[serde(default)]
    pub(crate)  is_final: bool,
}

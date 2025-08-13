/// Generated WIT bindings for transaction command
mod bindings {
    use super::TransactionAggregate;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "transaction-w",
    });

    export!(TransactionAggregate);
}

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use prost::Message;
use proto::{
    bank_event, AccountOpened, BankEvent, Transaction,
    TransactionDenied,
};

pub struct TransactionAggregate;

use bindings::exports::cosmonic::eventsourcing::*;
use bindings::cosmonic::eventsourcing::bank_account;

// Implement transaction-command interface
impl transaction_command::Guest for TransactionAggregate {
    fn serialize_command(command: transaction_command::Command) -> Result<Vec<u8>, String> {
        // Manual serialization since WIT records don't auto-implement Serde
        let json = format!(
            r#"{{"account_id":"{}","amount":{},"description":{}}}"#,
            command.account_id,
            command.amount,
            command.description.as_ref().map_or("null".to_string(), |d| format!(r#""{}""#, d))
        );
        Ok(json.into_bytes())
    }

    fn deserialize_command(command: Vec<u8>) -> Result<transaction_command::Command, String> {
        let _json_str = String::from_utf8(command)
            .map_err(|e| format!("Invalid UTF-8 in command: {e}"))?;
        
        // Simple JSON parsing - in a real implementation you'd use a proper JSON parser
        // For now, we'll create a minimal command
        Ok(transaction_command::Command {
            account_id: "default".to_string(),
            amount: -50, // Default transaction
            description: None,
        })
    }
}

// Implement transaction-state interface
impl transaction_state::Guest for TransactionAggregate {
    fn initial_state() -> transaction_state::State {
        transaction_state::State {
            id: String::new(),
            balance: 0,
            is_open: false,
            customer_id: String::new(),
            account_type: None,
        }
    }

    fn serialize_state(state: transaction_state::State) -> Result<Vec<u8>, String> {
        let proto_state = proto::BankState {
            balance: state.balance,
            id: state.id,
            is_open: state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<transaction_state::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        
        Ok(transaction_state::State {
            id: proto_state.id,
            balance: proto_state.balance,
            is_open: proto_state.is_open,
            customer_id: String::new(), // Not stored in proto for now
            account_type: None, // Not stored in proto for now
        })
    }
}

// Implement transaction main interface
impl transaction::Guest for TransactionAggregate {
    fn evolve(
        mut state: transaction_state::State,
        event: bank_account::Event,
    ) -> Result<transaction_state::State, String> {
        let bank_event = BankEvent::decode(event.data.as_slice())
            .map_err(|e| format!("Failed to decode event: {e}"))?;
            
        match bank_event.event.as_ref() {
            Some(bank_event::Event::Opened(AccountOpened { balance, id })) => {
                state.balance = i64::from(*balance);
                state.id = id.to_owned();
                state.is_open = true;
            }
            Some(bank_event::Event::Transaction(Transaction { amount })) => {
                state.balance += amount;
            }
            Some(bank_event::Event::Denied(TransactionDenied { amount: _ })) => {
                // could add a denied log to the account, or something
            }
            None => {}
        }
        Ok(state)
    }

    fn handle_command(
        state: transaction_state::State,
        command: transaction_command::Command,
    ) -> Result<Vec<bank_account::Event>, String> {
        if !state.is_open {
            return Err("Account is not open".to_string());
        }

        // Check if transaction would cause negative balance
        if state.balance.saturating_add(command.amount) < 0 {
            let denied = TransactionDenied {
                amount: command.amount,
            };

            let bank_event = BankEvent {
                event: Some(bank_event::Event::Denied(denied)),
            };

            Ok(vec![bank_account::Event {
                event_type: "transaction_denied".to_string(),
                data: bank_event.encode_to_vec(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                version: 1,
            }])
        } else {
            let transaction = Transaction {
                amount: command.amount,
            };

            let bank_event = BankEvent {
                event: Some(bank_event::Event::Transaction(transaction)),
            };

            Ok(vec![bank_account::Event {
                event_type: "transaction".to_string(),
                data: bank_event.encode_to_vec(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                version: 1,
            }])
        }
    }
}
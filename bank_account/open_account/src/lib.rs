/// Generated WIT bindings for open-account command
mod bindings {
    use super::TransactionAggregate;

    wit_bindgen::generate!({
        path: "../../wit",
        world: "open-account-w",
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

pub struct OpenAccountAggregate;

use bindings::exports::cosmonic::eventsourcing::*;
use bindings::cosmonic::eventsourcing::bank_account;

// Implement open-account-command interface
impl open_account_command::Guest for OpenAccountAggregate {
    fn serialize_command(command: open_account_command::Command) -> Result<Vec<u8>, String> {
        // Manual serialization since WIT records don't auto-implement Serde
        let json = format!(
            r#"{{"initial_balance":{},"customer_id":"{}","account_type":{}}}"#,
            command.initial_balance,
            command.customer_id,
            command.account_type.as_ref().map_or("null".to_string(), |t| format!(r#""{}""#, t))
        );
        Ok(json.into_bytes())
    }

    fn deserialize_command(command: Vec<u8>) -> Result<open_account_command::Command, String> {
        let json_str = String::from_utf8(command)
            .map_err(|e| format!("Invalid UTF-8 in command: {e}"))?;
        
        // Simple JSON parsing - in a real implementation you'd use a proper JSON parser
        // For now, we'll create a minimal command
        Ok(open_account_command::Command {
            initial_balance: 100, // Default for now
            customer_id: "default".to_string(),
            account_type: None,
        })
    }
}

// Implement open-account-state interface
impl open_account_state::Guest for OpenAccountAggregate {
    fn initial_state() -> open_account_state::State {
        open_account_state::State {
            id: String::new(),
            balance: 0,
            is_open: false,
            customer_id: String::new(),
            account_type: None,
        }
    }

    fn serialize_state(state: open_account_state::State) -> Result<Vec<u8>, String> {
        let proto_state = proto::BankState {
            balance: state.balance,
            id: state.id,
            is_open: state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<open_account_state::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        
        Ok(open_account_state::State {
            id: proto_state.id,
            balance: proto_state.balance,
            is_open: proto_state.is_open,
            customer_id: String::new(), // Not stored in proto for now
            account_type: None, // Not stored in proto for now
        })
    }
}

// Implement open-account main interface
impl open_account::Guest for OpenAccountAggregate {
    fn rehydrate(events: Vec<bank_account::Event>) -> Result<open_account_state::State, String> {
        let mut state = open_account_state::State {
            id: String::new(),
            balance: 0,
            is_open: false,
            customer_id: String::new(),
            account_type: None,
        };

        for event in events {
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
        }
        Ok(state)
    }

    fn handle_open_account(
        state: open_account_state::State,
        command: open_account_command::Command,
    ) -> Result<Vec<bank_account::Event>, String> {
        if state.is_open {
            return Err("Account is already open".to_string());
        }

        let account_opened = AccountOpened {
            balance: command.initial_balance,
            id: uuid::Uuid::now_v7().to_string(),
        };

        let bank_event = BankEvent {
            event: Some(bank_event::Event::Opened(account_opened)),
        };

        Ok(vec![bank_account::Event {
            event_type: "account_opened".to_string(),
            data: bank_event.encode_to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            version: 1,
        }])
    }
}
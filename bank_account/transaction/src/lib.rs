/// Generated WIT bindings for transaction command
mod bindings {
    use super::TransactionAggregate;

    wit_bindgen::generate!({
        path: "./wit",
        world: "transaction-w",
    });

    export!(TransactionAggregate);
}

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use prost::Message;
use proto::{
    bank_event, BankEvent, BankState, Transaction,
    TransactionDenied,
};

/// Transaction Command - specific to this handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TransactionCommand {
    pub amount: i64,
}

impl From<BankEvent> for shared_types::Event {
    fn from(value: BankEvent) -> Self {
        shared_types::Event::new(value)
    }
}
impl From<bank_event::Event> for shared_types::Event {
    fn from(value: bank_event::Event) -> Self {
        shared_types::Event::new(BankEvent { event: Some(value) })
    }
}
impl From<BankState> for transaction::State {
    fn from(value: BankState) -> Self {
        transaction::State::new(value)
    }
}
impl From<TransactionCommand> for transaction::Command {
    fn from(value: TransactionCommand) -> Self {
        transaction::Command::new(value)
    }
}

pub struct TransactionAggregate;

use bindings::exports::cosmonic::eventsourcing::*;

impl shared_types::GuestEvent for BankEvent {}
impl shared_types::Guest for TransactionAggregate {
    type Event = BankEvent;

    fn serialize_event(event: shared_types::Event) -> Result<Vec<u8>, String> {
        Ok(event.into_inner::<BankEvent>().encode_to_vec())
    }

    fn deserialize_event(event: Vec<u8>) -> Result<shared_types::Event, String> {
        Ok(BankEvent::decode(event.as_slice())
            .map_err(|e| format!("Event deserialization failed: {e}"))?
            .into())
    }
}

impl transaction::GuestCommand for TransactionCommand {}
impl transaction::GuestState for BankState {}
impl transaction::Guest for TransactionAggregate {
    type Command = TransactionCommand;
    type State = BankState;

    fn serialize_command(
        command: transaction::Command,
    ) -> Result<Vec<u8>, String> {
        let cmd = command.into_inner::<TransactionCommand>();
        // JSON serialization for this specific command
        serde_json::to_vec(&cmd).map_err(|e| format!("Command serialization failed: {e}"))
    }

    fn deserialize_command(
        command: Vec<u8>,
    ) -> Result<transaction::Command, String> {
        let cmd: TransactionCommand = serde_json::from_slice(&command)
            .map_err(|e| format!("Command deserialization failed: {e}"))?;
        Ok(cmd.into())
    }

    fn serialize_state(state: transaction::State) -> Result<Vec<u8>, String> {
        let bank_state = state.into_inner::<BankState>();
        let proto_state = proto::BankState {
            balance: bank_state.balance,
            id: bank_state.id,
            is_open: bank_state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<transaction::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        let bank_state = BankState {
            balance: proto_state.balance,
            id: proto_state.id,
            is_open: proto_state.is_open,
        };
        Ok(transaction::State::new(bank_state))
    }

    fn rehydrate(events: Vec<shared_types::Event>) -> Result<transaction::State, String> {
        let mut state = BankState::default();
        for e in events {
            match e.get::<BankEvent>().event.as_ref() {
                Some(bank_event::Event::Opened(proto::AccountOpened { balance, id })) => {
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
        Ok(state.into())
    }

    fn handle_transaction(
        state: transaction::State,
        command: transaction::Command,
    ) -> Result<Vec<shared_types::Event>, String> {
        let bank_state: &BankState = state.get();
        if !bank_state.is_open {
            return Err("Account is not open".to_string());
        }

        let cmd = command.into_inner::<TransactionCommand>();
        
        // Check if transaction would cause negative balance
        if bank_state.balance.saturating_sub(cmd.amount) < 0 {
            Ok(vec![bank_event::Event::Denied(TransactionDenied {
                amount: cmd.amount,
            })
            .into()])
        } else {
            Ok(vec![
                bank_event::Event::Transaction(Transaction { amount: cmd.amount }).into()
            ])
        }
    }
} 
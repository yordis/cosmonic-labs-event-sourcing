/// Generated WIT bindings for open-account command
mod bindings {
    use super::OpenAccountAggregate;

    wit_bindgen::generate!({
        path: "./wit",
        world: "open-account-w",
    });

    export!(OpenAccountAggregate);
}

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use prost::Message;
use proto::{
    bank_event, AccountOpened, BankEvent, BankState, Transaction,
    TransactionDenied,
};

/// Open Account Command - specific to this handler
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenAccountCommand {
    pub initial_balance: u32,
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
impl From<BankState> for open_account::State {
    fn from(value: BankState) -> Self {
        open_account::State::new(value)
    }
}
impl From<OpenAccountCommand> for open_account::Command {
    fn from(value: OpenAccountCommand) -> Self {
        open_account::Command::new(value)
    }
}

pub struct OpenAccountAggregate;

use bindings::exports::cosmonic::eventsourcing::*;

impl shared_types::GuestEvent for BankEvent {}
impl shared_types::Guest for OpenAccountAggregate {
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

impl open_account::GuestCommand for OpenAccountCommand {}
impl open_account::GuestState for BankState {}
impl open_account::Guest for OpenAccountAggregate {
    type Command = OpenAccountCommand;
    type State = BankState;

    fn serialize_command(
        command: open_account::Command,
    ) -> Result<Vec<u8>, String> {
        let cmd = command.into_inner::<OpenAccountCommand>();
        // Simple JSON serialization for this specific command
        serde_json::to_vec(&cmd).map_err(|e| format!("Command serialization failed: {e}"))
    }

    fn deserialize_command(
        command: Vec<u8>,
    ) -> Result<open_account::Command, String> {
        let cmd: OpenAccountCommand = serde_json::from_slice(&command)
            .map_err(|e| format!("Command deserialization failed: {e}"))?;
        Ok(cmd.into())
    }

    fn serialize_state(state: open_account::State) -> Result<Vec<u8>, String> {
        let bank_state = state.into_inner::<BankState>();
        let proto_state = proto::BankState {
            balance: bank_state.balance,
            id: bank_state.id,
            is_open: bank_state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<open_account::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        let bank_state = BankState {
            balance: proto_state.balance,
            id: proto_state.id,
            is_open: proto_state.is_open,
        };
        Ok(open_account::State::new(bank_state))
    }

    fn rehydrate(events: Vec<shared_types::Event>) -> Result<open_account::State, String> {
        let mut state = BankState::default();
        for e in events {
            match e.get::<BankEvent>().event.as_ref() {
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
        Ok(state.into())
    }

    fn handle_open_account(
        state: open_account::State,
        command: open_account::Command,
    ) -> Result<Vec<shared_types::Event>, String> {
        let bank_state: &BankState = state.get();
        if bank_state.is_open {
            return Err("Account is already open".to_string());
        }

        let cmd = command.into_inner::<OpenAccountCommand>();
        Ok(vec![bank_event::Event::Opened(AccountOpened {
            balance: cmd.initial_balance,
            id: uuid::Uuid::now_v7().to_string(),
        })
        .into()])
    }
}
/// Generated WIT bindings for aggregate (legacy/compatibility)
mod bindings {
    use super::Aggregate;

    wit_bindgen::generate!({
        path: "./wit",
        world: "aggregate-w",
    });

    export!(Aggregate);
}

pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/banking.rs"));
}

use prost::Message;
use proto::{
    bank_command, bank_event, AccountOpened, BankCommand, BankEvent, BankState, Transaction,
    TransactionDenied,
};

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
impl From<BankState> for aggregate::State {
    fn from(value: BankState) -> Self {
        aggregate::State::new(value)
    }
}
impl From<BankCommand> for aggregate::Command {
    fn from(value: BankCommand) -> Self {
        aggregate::Command::new(value)
    }
}

pub struct Aggregate;

use bindings::exports::cosmonic::eventsourcing::*;

impl shared_types::GuestEvent for BankEvent {}
impl shared_types::Guest for Aggregate {
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

impl aggregate::GuestCommand for BankCommand {}
impl aggregate::GuestState for BankState {}
impl aggregate::Guest for Aggregate {
    type Command = BankCommand;
    type State = BankState;

    fn serialize_command(
        command: aggregate::Command,
    ) -> Result<Vec<u8>, String> {
        Ok(command.into_inner::<BankCommand>().encode_to_vec())
    }

    fn deserialize_command(
        command: Vec<u8>,
    ) -> Result<aggregate::Command, String> {
        Ok(BankCommand::decode(command.as_slice())
            .map_err(|e| format!("Command deserialization failed: {e}"))?
            .into())
    }

    fn serialize_state(state: aggregate::State) -> Result<Vec<u8>, String> {
        let bank_state = state.into_inner::<BankState>();
        let proto_state = proto::BankState {
            balance: bank_state.balance,
            id: bank_state.id,
            is_open: bank_state.is_open,
        };
        Ok(proto_state.encode_to_vec())
    }

    fn deserialize_state(state: Vec<u8>) -> Result<aggregate::State, String> {
        let proto_state = proto::BankState::decode(state.as_slice())
            .map_err(|e| format!("State deserialization failed: {e}"))?;
        let bank_state = BankState {
            balance: proto_state.balance,
            id: proto_state.id,
            is_open: proto_state.is_open,
        };
        Ok(aggregate::State::new(bank_state))
    }

    fn rehydrate(events: Vec<shared_types::Event>) -> Result<aggregate::State, String> {
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

    fn handle(
        state: aggregate::State,
        command: aggregate::Command,
    ) -> Result<Vec<shared_types::Event>, String> {
        let bank_state: &BankState = state.get();
        let bank_command = command.into_inner::<BankCommand>();
        match bank_command.command {
            Some(bank_command::Command::Transaction(amount)) => {
                if bank_state.balance.saturating_sub(amount) < 0 {
                    Ok(vec![bank_event::Event::Denied(TransactionDenied {
                        amount,
                    })
                    .into()])
                } else {
                    Ok(vec![
                        bank_event::Event::Transaction(Transaction { amount }).into()
                    ])
                }
            }
            Some(bank_command::Command::OpenAccount(initial_balance)) => {
                if bank_state.is_open {
                    return Err("Account is already open".to_string());
                }
                Ok(vec![bank_event::Event::Opened(AccountOpened {
                    balance: initial_balance,
                    id: uuid::Uuid::now_v7().to_string(),
                })
                .into()])
            }
            None => Ok(vec![]),
        }
    }
}

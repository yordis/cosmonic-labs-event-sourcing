/// Generated WIT bindings
mod bindings {
    use super::EventSourcer;
    wit_bindgen::generate!({
        path: "../wit",
        world: "event-sourcer-w",
    });

    export!(EventSourcer);
}

use bindings::cosmonic::eventsourcing::*;
use bindings::exports::cosmonic::eventsourcing::*;

struct EventSourcer;

impl event_sourcer::Guest for EventSourcer {
    fn get_events(aggregate_id: String) -> Result<Vec<bank_account::Event>, String> {
        let bytes = event_store::get_events(&aggregate_id)?;
        let mut events = Vec::with_capacity(bytes.len());
        for event_bytes in bytes {
            events.push(
                bank_account::deserialize_event(&event_bytes)
                    .map_err(|e| format!("failed to deserialize event: {e}"))?,
            );
        }

        Ok(events)
    }

    fn append(aggregate_id: String, new_events: Vec<bank_account::Event>) -> Result<Vec<Vec<u8>>, String> {
        let mut all_events = Vec::with_capacity(new_events.len());

        for event in new_events {
            let event_bytes = bank_account::serialize_event(&event)
                .map_err(|e| format!("Failed to serialize event: {e}"))?;
            event_store::append_event(&aggregate_id, &event_bytes)?;
            all_events.push(event_bytes);
        }

        Ok(all_events)
    }

    fn handle_command(
        aggregate_id: String,
        command: Vec<u8>,
    ) -> Result<Vec<bank_account::Event>, String> {
        let events_bytes = event_store::get_events(&aggregate_id)?;
        let mut events = Vec::with_capacity(events_bytes.len());

        for event_bytes in events_bytes {
            events.push(bank_account::deserialize_event(&event_bytes)?);
        }

        // For now, return empty events - this would need to be implemented
        // based on the specific aggregate logic
        Ok(vec![])
    }
}

/// Generated WIT bindings
mod bindings {
    use super::EventSourcer;
    wit_bindgen::generate!({
        path: "../wit",
        world: "http-api-gateway-w",
        generate_all,
    });

    export!(EventSourcer);
}

use bindings::cosmonic::eventsourcing::*;
use bindings::exports::wasi::http::incoming_handler::Guest;

use crate::bindings::wasi::http::types::{
    Fields, IncomingRequest, OutgoingBody, OutgoingResponse, ResponseOutparam,
};

struct EventSourcer;

impl Guest for EventSourcer {
    fn handle(request: IncomingRequest, response_out: ResponseOutparam) {
        let body = request.consume().expect("to get body");
        let input_stream = body.stream().expect("to get stream");
        let bytes = input_stream
            .blocking_read(4096)
            .expect("to read some bytes");
        let events = event_sourcer::handle_command("foobar", &bytes).expect("to handle");
        let _res = event_sourcer::append("foobar", &events);

        let res = OutgoingResponse::new(Fields::new());
        let body = res.body().expect("to get outgoing body");
        let out = body.write().expect("to get write stream");

        ResponseOutparam::set(response_out, Ok(res));

        out.blocking_write_and_flush(b"you did it")
            .expect("to write bytes");
        drop(out);

        OutgoingBody::finish(body, None).expect("to finish body");
    }
}

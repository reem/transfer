use std::sync::Arc;

use eventual::Sender;

use rt::connection::Snapshot;
use rt::Metadata;

// FIXME: This should go away
use rt::connection::Response;

use prelude::*;

pub mod parser;

struct Request;

pub fn handle_connection(
    metadata: Metadata, handler: Arc<Box<Handler>>,
    snapshots: Stream<Snapshot, Error>, responses: Sender<Response, Error>) {

    let (requests_tx, requests_rx) = Stream::<Request, Error>::pair();

    snapshots.each(move |snapshot| {
        match snapshot {
            Snapshot::Head(head) => {

            },
            Snapshot::Body(_) => {}
        }
    }).fire();
}



use std::mem;

use crossbeam_channel::Sender;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::http::Method;

use crate::blinds::{BlindsAction, BlindsActionCause, BlindsTime};

pub fn run(action_tx: Sender<BlindsAction>, time_tx: Sender<BlindsTime>) {
    let mut server = EspHttpServer::new(&Configuration::default()).unwrap();

    server
        .fn_handler("/up", Method::Post, {
            let action_tx = action_tx.clone();
            move |request| {
                action_tx
                    .send(BlindsAction::MoveUp(BlindsActionCause::Manual))
                    .unwrap();
                request.into_ok_response()?.flush()
            }
        })
        .unwrap();

    server
        .fn_handler("/down", Method::Post, {
            move |request| {
                action_tx
                    .send(BlindsAction::MoveDown(BlindsActionCause::Manual))
                    .unwrap();
                request.into_ok_response()?.flush()
            }
        })
        .unwrap();

    server
        .fn_handler("/up_time", Method::Post, {
            let time_tx = time_tx.clone();
            move |mut request| -> anyhow::Result<_> {
                // parse time from body
                let (_headers, connection) = request.split();
                let mut buffer: [u8; 32] = [0; 32];
                let n = connection.read(&mut buffer)?;
                let time = serde_json::de::from_slice(&buffer[..n])?;

                time_tx.send(BlindsTime::SetUp(time)).unwrap();

                request.into_ok_response()?;
                Ok(())
            }
        })
        .unwrap();

    server
        .fn_handler("/down_time", Method::Post, {
            move |mut request| -> anyhow::Result<_> {
                // parse time from body
                let (_headers, connection) = request.split();
                let mut buffer: [u8; 32] = [0; 32];
                let n = connection.read(&mut buffer)?;
                let time = serde_json::de::from_slice(&buffer[..n])?;

                time_tx.send(BlindsTime::SetDown(time)).unwrap();

                request.into_ok_response()?;
                Ok(())
            }
        })
        .unwrap();

    // keep server running forever
    mem::forget(server);
}


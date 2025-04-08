use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use esp_idf_svc::timer::EspTimerService;
use time::ext::NumericalStdDuration;
use time::Time;

use crate::ui::UIEvent;
use crate::util;

const BLINDS_MOVEMENT_TIME: Duration = Duration::from_secs(5);

#[derive(Debug)]
pub enum BlindsActionCause {
    Manual,
    Timer,
}

#[derive(Debug)]
pub enum BlindsAction {
    MoveUp(BlindsActionCause),
    MoveDown(BlindsActionCause),
}

#[derive(Debug)]
pub enum BlindsTime {
    SetUp(Time),
    SetDown(Time),
}

pub fn action_timers(
    action_tx: Sender<BlindsAction>,
    up_time: impl AsRef<Mutex<Time>>,
    down_time: impl AsRef<Mutex<Time>>,
    ui_tx: Sender<UIEvent>,
    time_rx: Receiver<BlindsTime>,
) {
    let timer_service = EspTimerService::new().unwrap();

    // up timer action
    let up_timer = {
        let action_tx = action_tx.clone();
        timer_service
            .timer(move || {
                action_tx
                    .send(BlindsAction::MoveUp(BlindsActionCause::Timer))
                    .unwrap();
            })
            .unwrap()
    };

    // init up timer
    log::info!("Initializing up timer");
    let duration = util::duration_until(*up_time.as_ref().lock().unwrap());
    up_timer.after(duration).unwrap();

    // down timer action
    let down_timer = {
        timer_service
            .timer(move || {
                action_tx
                    .send(BlindsAction::MoveDown(BlindsActionCause::Timer))
                    .unwrap();
            })
            .unwrap()
    };

    // init down timer
    log::info!("Initializing down timer");
    let duration = util::duration_until(*down_time.as_ref().lock().unwrap());
    down_timer.after(duration).unwrap();

    for msg in time_rx {
        match msg {
            BlindsTime::SetUp(time) => {
                log::info!("Setting new up timer");
                ui_tx.send(UIEvent::SetUp).unwrap();

                // update up time
                *up_time.as_ref().lock().unwrap() = time;

                // set new up timer
                up_timer.after(duration).unwrap();
            }
            BlindsTime::SetDown(time) => {
                log::info!("Setting new down timer");
                ui_tx.send(UIEvent::SetDown).unwrap();

                // update down time
                *down_time.as_ref().lock().unwrap() = time;

                // set new down timer
                down_timer.after(duration).unwrap();
            }
        }
    }
}

pub fn action_handler(
    mut up_pin: impl embedded_hal::digital::OutputPin,
    mut down_pin: impl embedded_hal::digital::OutputPin,
    ui_tx: Sender<UIEvent>,
    time_tx: Sender<BlindsTime>,
    up_time: impl AsRef<Mutex<Time>>,
    down_time: impl AsRef<Mutex<Time>>,
    action_rx: Receiver<BlindsAction>,
) {
    for msg in action_rx {
        match msg {
            BlindsAction::MoveUp(cause) => {
                log::info!("Raising blinds.");
                ui_tx.send(UIEvent::MoveUp).unwrap();

                // raise blinds
                up_pin.set_high().unwrap();
                thread::sleep(200.std_milliseconds());
                up_pin.set_low().unwrap();

                if let BlindsActionCause::Timer = cause {
                    // set new up timer
                    time_tx
                        .send(BlindsTime::SetUp(*up_time.as_ref().lock().unwrap()))
                        .unwrap();
                }
            }
            BlindsAction::MoveDown(cause) => {
                log::info!("Lowering blinds.");
                ui_tx.send(UIEvent::MoveDown).unwrap();

                // lower blinds
                down_pin.set_high().unwrap();
                thread::sleep(200.std_milliseconds());
                down_pin.set_low().unwrap();

                if let BlindsActionCause::Timer = cause {
                    // set new down timer
                    time_tx
                        .send(BlindsTime::SetDown(*down_time.as_ref().lock().unwrap()))
                        .unwrap();
                }
            }
        }

        // wait for blinds to stop
        thread::sleep(BLINDS_MOVEMENT_TIME);
        log::info!("Blinds stopped");
        ui_tx.send(UIEvent::Stop).unwrap();
    }
}

use std::sync::{Arc, Mutex};
use std::thread;

use crossbeam_channel::{bounded, select, tick};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::EspSntp;
use esp_idf_svc::wifi::EspWifi;
use time::ext::NumericalStdDuration;
use time::Time;

use self::blinds::{BlindsAction, BlindsTime};
use self::ui::UIEvent;

mod blinds;
mod server;
mod timezone;
mod ui;
mod util;
mod wifi;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;

    // init display
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio21;
    let scl = peripherals.pins.gpio20;
    let mut display = ui::setup_display(i2c, sda, scl);

    // connect wifi
    let sysloop = EspSystemEventLoop::take().unwrap();
    let mut esp_wifi = EspWifi::new(
        peripherals.modem,
        sysloop.clone(),
        Some(EspDefaultNvsPartition::take().unwrap()),
    )
    .unwrap();

    wifi::connect(
        &mut esp_wifi,
        sysloop,
        env!("WIFI_SSID"),
        env!("WIFI_PASSWORD"),
    );

    // start time sync
    let _sntp = EspSntp::new_default().unwrap();

    // Europe/Berlin
    timezone::set_timezone("CET-1CEST,M3.5.0/2,M10.5.0/3");

    let (action_tx, action_rx) = bounded::<BlindsAction>(1);
    let (time_tx, time_rx) = bounded::<BlindsTime>(1);
    let (ui_tx, ui_event_rx) = bounded::<UIEvent>(1);

    let up_time = Arc::new(Mutex::new(Time::from_hms(7, 0, 0)?));
    let down_time = Arc::new(Mutex::new(Time::from_hms(18, 0, 0)?));

    thread::spawn({
        let action_tx = action_tx.clone();
        let time_tx = time_tx.clone();
        move || {
            server::run(action_tx, time_tx);
        }
    });

    thread::spawn({
        let action_tx = action_tx.clone();
        let ui_tx = ui_tx.clone();
        let up_time = up_time.clone();
        let down_time = down_time.clone();
        move || {
            blinds::action_timers(action_tx, up_time, down_time, ui_tx, time_rx);
        }
    });

    thread::spawn({
        let up_pin = PinDriver::output(peripherals.pins.gpio0).unwrap();
        let down_pin = PinDriver::output(peripherals.pins.gpio1).unwrap();

        let down_time = down_time.clone();
        let up_time = up_time.clone();
        move || {
            blinds::action_handler(
                up_pin, down_pin, ui_tx, time_tx, up_time, down_time, action_rx,
            );
        }
    });

    let mut blinds_action = None;

    let time_rx = tick(10.std_seconds());

    ui::draw(
        *up_time.lock().unwrap(),
        *down_time.lock().unwrap(),
        blinds_action,
        &mut display,
    )
    .unwrap();
    display.flush().unwrap();

    loop {
        select! {
            recv(time_rx) -> _ => {}
            recv(ui_event_rx) -> msg => match msg? {
                UIEvent::MoveUp => blinds_action = Some("Raising blinds..."),
                UIEvent::MoveDown => blinds_action = Some("Lowering blinds..."),
                UIEvent::Stop => blinds_action = None,
                UIEvent::SetUp | UIEvent::SetDown => {}
            }
        };

        display.clear_buffer();
        ui::draw(
            *up_time.lock().unwrap(),
            *down_time.lock().unwrap(),
            blinds_action,
            &mut display,
        )
        .unwrap();
        display.flush().unwrap();
    }
}

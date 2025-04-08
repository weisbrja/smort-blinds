use time::{OffsetDateTime, Time};
use time::ext::NumericalDuration;

pub fn duration_until(time: Time) -> std::time::Duration {
    let now = OffsetDateTime::now_local().unwrap();
    let mut then = now.replace_time(time);
    if now.time() > time {
        then += 1.days();
    }
    let duration = then - now;
    log::info!("Duration from {now} until {time} is {duration}");
    duration.try_into().unwrap()
}

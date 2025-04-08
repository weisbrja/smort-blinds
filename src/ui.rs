use embedded_graphics::mono_font::iso_8859_1::FONT_5X8;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Circle, PrimitiveStyle, Triangle};
use embedded_graphics::text::{Alignment, Baseline, Text, TextStyleBuilder};
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};
use esp_idf_svc::hal::i2c::{I2c, I2cConfig, I2cDriver};
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::*;
use ssd1306::mode::BufferedGraphicsMode;
use ssd1306::prelude::*;
use ssd1306::{I2CDisplayInterface, Ssd1306};
use time::{OffsetDateTime, Time};

#[derive(Debug)]
pub enum UIEvent {
    MoveUp,
    MoveDown,
    Stop,
    SetUp,
    SetDown,
}

pub fn setup_display<'d>(
    i2c: impl Peripheral<P = impl I2c> + 'd,
    sda: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
    scl: impl Peripheral<P = impl InputPin + OutputPin> + 'd,
) -> Ssd1306<I2CInterface<I2cDriver<'d>>, DisplaySize128x32, BufferedGraphicsMode<DisplaySize128x32>>
{
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(interface, DisplaySize128x32, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init().unwrap();

    display.clear_buffer();
    display.flush().unwrap();

    display
}

pub fn draw<D: DrawTarget<Color = BinaryColor>>(
    up_time: Time,
    down_time: Time,
    blinds_action: Option<impl AsRef<str>>,
    display: &mut D,
) -> Result<(), D::Error> {
    let font = MonoTextStyleBuilder::new()
        .font(&FONT_5X8)
        .text_color(BinaryColor::On)
        .build();
    let fill = PrimitiveStyle::with_fill(BinaryColor::On);
    let stroke = PrimitiveStyle::with_stroke(BinaryColor::On, 1);

    let time_format = time::format_description::parse_owned::<2>("[hour]:[minute]").unwrap();

    let up_symbol =
        Triangle::new(Point::new(2, 1), Point::new(0, 6), Point::new(4, 6)).into_styled(fill);
    up_symbol.draw(display)?;

    let clock_symbol = Circle::new(Point::new(95, 1), 6).into_styled(stroke);

    let up_time = up_time.format(&time_format).unwrap();
    let up_time_ui = Text::with_baseline(&up_time, Point::new(7, 0), font, Baseline::Top);
    up_time_ui.draw(display)?;

    let down_symbol = Triangle::new(Point::new(0, 1), Point::new(2, 6), Point::new(4, 1))
        .translate_mut(Point::new(40, 0))
        .into_styled(fill);
    down_symbol.draw(display)?;

    let down_time = down_time.format(&time_format).unwrap();
    let down_time_ui = Text::with_baseline(&down_time, Point::new(47, 0), font, Baseline::Top);
    down_time_ui.draw(display)?;

    let current_time = OffsetDateTime::now_local()
        .unwrap()
        .format(&time_format)
        .unwrap();
    let current_time_ui = Text::with_text_style(
        &current_time,
        Point::new(127, 0),
        font,
        TextStyleBuilder::new()
            .baseline(Baseline::Top)
            .alignment(Alignment::Right)
            .build(),
    );
    current_time_ui.draw(display)?;

    clock_symbol.draw(display)?;

    if let Some(blinds_action) = blinds_action {
        let blinds_action_ui = Text::with_baseline(
            blinds_action.as_ref(),
            Point::new(0, 8),
            font,
            Baseline::Top,
        );
        blinds_action_ui.draw(display)?;
    }

    Ok(())
}

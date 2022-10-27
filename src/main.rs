#![no_std]
#![no_main]

#[cfg(feature="esp32")]
use esp32_hal as hal;
#[cfg(feature="esp32s2")]
use esp32s2_hal as hal;
#[cfg(feature="esp32s3")]
use esp32s3_hal as hal;
#[cfg(feature="esp32c3")]
use esp32c3_hal as hal;

use hal::{
    clock::ClockControl,
    pac::Peripherals,
    gpio_types::*, 
    gpio::*,
    prelude::*,
    spi::{dma::WithDmaSpi2, Spi, SpiMode},
    systimer::SystemTimer,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};


use keypad2::{Keypad, Columns, Rows};

use ili9341::{DisplaySize240x320, Ili9341, Orientation};

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use embedded_graphics::image::Image;
use embedded_graphics::geometry::*;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyleBuilder};

use profont::{PROFONT_24_POINT, PROFONT_18_POINT};



#[cfg(feature="xtensa-lx-rt")]
use xtensa_lx_rt::entry;
#[cfg(feature="riscv-rt")]
use riscv_rt::entry;

use esp_println::println;
use esp_backtrace as _;


/* Some stuff for correct orientation and color on ILI9341 */
pub enum KalugaOrientation {
    Portrait,
    PortraitFlipped,
    Landscape,
    LandscapeVericallyFlipped,
    LandscapeFlipped,
}

impl ili9341::Mode for KalugaOrientation {
    fn mode(&self) -> u8 {
        match self {
            Self::Portrait => 0,
            Self::LandscapeVericallyFlipped => 0x20,
            Self::Landscape => 0x20 | 0x40,
            Self::PortraitFlipped => 0x80 | 0x40,
            Self::LandscapeFlipped => 0x80 | 0x20,
        }
    }

    fn is_landscape(&self) -> bool {
        matches!(self, Self::Landscape | Self::LandscapeFlipped | Self::LandscapeVericallyFlipped)
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum Event {
    Pressed,
    Released,
    Nothing,
}
pub struct Button<T> {
    button: T,
    pressed: bool,
}
impl<T: ::embedded_hal::digital::v2::InputPin<Error = core::convert::Infallible>> Button<T> {
    pub fn new(button: T) -> Self {
        Button {
            button,
            pressed: true,
        }
    }
    pub fn check(&mut self){
        self.pressed = !self.button.is_low().unwrap();
    }

    pub fn poll(&mut self, delay :&mut Delay) -> Event {
        let pressed_now = !self.button.is_low().unwrap();
        if !self.pressed  &&  pressed_now
        {
            delay.delay_ms(30 as u32);
            self.check();
            if !self.button.is_low().unwrap() {
                Event::Pressed
            }
            else {
                Event::Nothing
            }
        }
        else if self.pressed && !pressed_now{
            delay.delay_ms(30 as u32);
            self.check();
            if self.button.is_low().unwrap()
            {
                Event::Released
            }
            else {
                Event::Nothing
            }
        }
        else{
            Event::Nothing
        }
        
    }
}



#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    #[cfg(any(feature = "esp32"))]
    let mut system = peripherals.DPORT.split();
    #[cfg(any(feature = "esp32s2", feature = "esp32s3", feature = "esp32c3"))]
    let mut system = peripherals.SYSTEM.split();

    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Disable the RTC and TIMG watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;
    
    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

 
    let mut delay = Delay::new(&clocks);
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut row1 = io.pins.gpio6.into_pull_up_input();
    let mut row2 = io.pins.gpio5.into_pull_up_input();
    let mut row3 = io.pins.gpio4.into_pull_up_input();
    let mut row4 = io.pins.gpio3.into_pull_up_input();

    let mut col1 = io.pins.gpio2.into_open_drain_output();
    let mut col2 = io.pins.gpio1.into_open_drain_output();
    let mut col3 = io.pins.gpio0.into_open_drain_output();

    let rows = (
        io.pins.gpio6.into_pull_up_input(),
        io.pins.gpio5.into_pull_up_input(),
        io.pins.gpio4.into_pull_up_input(),
        io.pins.gpio3.into_pull_up_input(),
    );

    let cols = ( 
        col1,
        col2,
        col3,
    );

    let mut keypad = Keypad::new(rows, cols);
    

    loop {
        let key = keypad.read_char(&mut delay);

        if key != ' '
        {
            println!("{}", key);
        }
    }
}
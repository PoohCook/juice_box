

use ws2812_spi as ws2812;

use crate::hal::rcc::*;
use crate::hal::pac::*;
use crate::hal::gpio::{NoPin, Pin};
use crate::hal::prelude::*;
use crate::hal::spi::Spi;
use crate::ws2812::prerendered::Ws2812;

use smart_leds::{SmartLedsWrite, RGB8};
// use rtt_target::{rprintln, rtt_init_print};

pub const LED_NUM: usize = 20;

pub const COLORS: [RGB8; 8] = [
        RGB8::new(0x00, 0x00, 0x00),
        RGB8::new(0x3f, 0x00, 0x00),
        RGB8::new(0x3f, 0x3f, 0x00),
        RGB8::new(0x00, 0x3f, 0x00),
        RGB8::new(0x00, 0x3f, 0x3f),
        RGB8::new(0x00, 0x00, 0x3f),
        RGB8::new(0x3f, 0x00, 0x3f),
        RGB8::new(0x3f, 0x3f, 0x3f),
    ];

pub struct LightPorts<'a> {
    led_data: [RGB8; LED_NUM],
    ws: Ws2812<'a, Spi<SPI1>>,
}

impl <'a> LightPorts<'a> {
    pub fn new(
        pa5: Pin<'A', 5>,
        pa7: Pin<'A', 7>,
        spi: SPI1,
        buffer: &'a mut [u8; (LED_NUM * 12) + 30],
        clocks: &Clocks,
    ) -> Self {
        // SPI1 with 3Mhz
        let spi: Spi<SPI1> = Spi::new(
            spi,
            (pa5.into_alternate(), NoPin::new(), pa7.into_alternate()),
            ws2812::MODE,
            3_000_000.Hz(),
            clocks,
        );

        let data = [RGB8::new(0x00, 0x00, 0x00); LED_NUM];

        // Create Ws2812 instance with the mutable reference to the buffer
        let ws = Ws2812::new(spi, buffer);

        // Return the LightPorts instance
        Self {
            led_data: data,
            ws
        }
    }

    pub fn set_bar(&mut self, bank: u8, color: RGB8) -> Result<(), &'static str>{
        let bank = bank as usize;
        if bank >= 4 {
            return Err("bar index out of range")
        }

        let index = bank * 3;
        for i in 0..3 {
            self.led_data[index + i] = color;
        }

        Ok(())
    }

    pub fn set_button(&mut self, bank: u8, button: usize, color: RGB8) -> Result<(), &'static str>{
        let bank = bank as usize;
        if bank >= 4 {
            return Err("bar index out of range")
        }

        let index = (bank * 2) + button + 12;
        self.led_data[index] = color;

        Ok(())
    }

    pub fn refresh(&mut self) -> Result<(), stm32f4xx_hal::spi::Error> {
        self.ws.write(self.led_data.iter().cloned())

    }

}

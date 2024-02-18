




#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use ws2812_spi as ws2812;

// use crate::hal::delay::Delay;
use crate::hal::pac;
use crate::hal::gpio::{NoPin, Pin};
use crate::hal::prelude::*;
use crate::hal::spi::{Spi};
// use crate::ws2812::Ws2812;
use crate::ws2812::prerendered::Ws2812;

use smart_leds::{gamma, SmartLedsWrite, RGB8, hsv::Hsv, hsv::hsv2rgb};
use rtt_target::{rprintln, rtt_init_print};

mod test_points;
use test_points::{*};

mod display;
use display::*;


struct LightPorts<'a> {
    led_data: [RGB8; 20],
    ws: Ws2812<'a, Spi<pac::SPI1>>,
}

impl <'a> LightPorts<'a> {
    fn new(
        pa5: Pin<'A', 5>,
        pa7: Pin<'A', 7>,
        spi: pac::SPI1,
        buffer: &'a mut [u8; (20 * 12) + 30],
        clocks: &hal::rcc::Clocks,
    ) -> Self {
        // SPI1 with 3Mhz
        let spi: Spi<pac::SPI1> = Spi::new(
            spi,
            (pa5.into_alternate(), NoPin::new(), pa7.into_alternate()),
            ws2812::MODE,
            3_000_000.Hz(),
            clocks,
        );

        const LED_NUM: usize = 20;
        let data = [RGB8::new(0x00, 0x00, 0x00); LED_NUM];

        // Create Ws2812 instance with the mutable reference to the buffer
        let ws = Ws2812::new(spi, buffer);

        // Return the LightPorts instance
        Self {
            led_data: data,
            ws
        }
    }

    fn get_iter(&mut self) -> core::slice::Iter<'_, RGB8> {
        self.led_data.iter()
    }

    fn set_bar(&mut self, bar: usize, color: RGB8) -> Result<(), &'static str>{
        if bar >= 4 {
            return Err("bar index out of range")
        }

        let mut index = bar * 3;
        for i in 0..3 {
            self.led_data[index + i] = color;
        }

        Ok(())
    }

    fn set_button(&mut self, bar: usize, button: usize, color: RGB8) -> Result<(), &'static str>{
        if bar >= 4 {
            return Err("bar index out of range")
        }

        let mut index = (bar * 2) + button + 12;
        self.led_data[index] = color;


        Ok(())
    }

    fn refresh(&mut self) {
        let res = self.ws.write(self.led_data.iter().cloned());
        rprintln!("result: {:?}", res);

    }

}

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();
    let clocks: hal::rcc::Clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(
        gpioc.pc0,
        gpioc.pc1,
        gpioc.pc2,
        gpioc.pc3,
        gpioc.pc4,
        gpioc.pc5,
        gpioc.pc6,
        gpioc.pc7,
    );
    test_point.reset_all();

    // Configure SPI peripheral
    let mut display = TM1638::new(
        gpioc.pc8,
        gpioc.pc9,
        gpioc.pc10,
    );

    display.initialize(7);
    display.set_brightness(7);
    display.display_num(1, 0);
    display.display_num(2, 0);
    display.display_num(3, 0);
    display.display_num(4, 0);

    let mut buffer = [0u8; (20 * 12) + 30];
    let mut lights = LightPorts::new(gpioa.pa5, gpioa.pa7, dp.SPI1, &mut buffer, &clocks);

    set!(test_point, 4);
    let mut cur_count = 0;

    let mut butts = [0,0,0,0];

    let colors = [
        RGB8::new(0x00, 0x00, 0x00),
        RGB8::new(0x3f, 0x00, 0x00),
        RGB8::new(0x3f, 0x3f, 0x00),
        RGB8::new(0x00, 0x3f, 0x00),
        RGB8::new(0x00, 0x3f, 0x3f),
        RGB8::new(0x00, 0x00, 0x3f),
        RGB8::new(0x3f, 0x00, 0x3f),
        RGB8::new(0x3f, 0x3f, 0x3f),
    ];

    loop {
        cur_count += 1;
        if cur_count > 99 {
            cur_count = 0;
        }

        set!(test_point, 5);
        lights.refresh();

        test_point.reset_all();
        set!(test_point, 6);
        // let butts = display.read_buttons();
        let key_event = display.get_key_events();

        match key_event {
            Some(ev) => {
                match ev {
                    KeyEvent::KeyDown { key: 1 } => {
                        butts[0] += 1;
                        display.display_num(1, butts[0]);
                        lights.set_bar(0, colors[(butts[0]%8) as usize]).unwrap();
                        lights.set_button(0, 0, colors[3]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 3 } => {
                        butts[1] += 1;
                        display.display_num(2, butts[1]);
                        lights.set_bar(1, colors[(butts[1]%8) as usize]).unwrap();
                        lights.set_button(1, 0, colors[3]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 5 } => {
                        butts[2] += 1;
                        display.display_num(3, butts[2]);
                        lights.set_bar(2, colors[(butts[2]%8) as usize]).unwrap();
                        lights.set_button(2, 0, colors[3]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 7 } => {
                        butts[3] += 1;
                        display.display_num(4, butts[3]);
                        lights.set_bar(3, colors[(butts[3]%8) as usize]).unwrap();
                        lights.set_button(3, 0, colors[3]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 2 } => {
                        lights.set_button(0, 1, colors[5]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 4 } => {
                        lights.set_button(1, 1, colors[5]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 6 } => {
                        lights.set_button(2, 1, colors[5]).unwrap();
                    },
                    KeyEvent::KeyDown { key: 8 } => {
                        lights.set_button(3, 1, colors[5]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 1 } => {
                        lights.set_button(0, 0, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 2 } => {
                        lights.set_button(0, 1, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 3 } => {
                        lights.set_button(1, 0, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 4 } => {
                        lights.set_button(1, 1, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 5 } => {
                        lights.set_button(2, 0, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 6 } => {
                        lights.set_button(2, 1, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 7 } => {
                        lights.set_button(3, 0, colors[0]).unwrap();
                    },
                    KeyEvent::KeyUp { key: 8 } => {
                        lights.set_button(3, 1, colors[0]).unwrap();
                    },
                    _ => {}
                }
            }
            None => {}
        }

        cortex_m::asm::delay(1_000_000);

    }}

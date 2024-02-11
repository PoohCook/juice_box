#![deny(unsafe_code)]
#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _;

use cortex_m_rt::entry;
use stm32f4xx_hal::{
    gpio::*, pac, prelude::*
};

mod test_points;
use test_points::{*};


struct TM1638 {
    stb: Pin<'C', 8, Output>,
    clk: Pin<'C', 9, Output>,
    dio: DynamicPin<'C', 10>
}

impl TM1638 {
    pub fn new(
        pc8: Pin<'C', 8>,
        pc9: Pin<'C', 9>,
        pc10: Pin<'C', 10>,

    ) -> Self {
        let mut me = TM1638{
            stb: pc8.into_push_pull_output(),
            clk: pc9.into_push_pull_output(),
            dio: pc10.into_dynamic()
        };

        me.stb.set_high();
        me.clk.set_high();
        me.dio.make_pull_up_input();

        me
    }

    fn write_byte(&mut self, data: u8){
        self.dio.make_push_pull_output();

        for i in 0..8 {
            self.clk.set_low();
            let mask: u8 = 0x01 << i;
            if data & mask == 0 {
                self.dio.set_low();
            } else {
                self.dio.set_low();
            }

            cortex_m::asm::delay(5);
            self.clk.set_high();
            cortex_m::asm::delay(5);
        }

        self.dio.make_pull_up_input();

    }

    pub fn write_display(&mut self, data: &[u8; 8]) {
        self.stb.set_low();
        cortex_m::asm::delay(5);
        self.write_byte(0x40);

        for d in data {
            self.write_byte(*d);
            self.write_byte(0);
        }

        self.stb.set_high();
        cortex_m::asm::delay(5);
    }

    pub fn write_command(&mut self, command: u8){

        self.stb.set_low();
        cortex_m::asm::delay(5);
        self.write_byte(command);
        self.stb.set_high();
        cortex_m::asm::delay(5);

    }

}


#[allow(clippy::empty_loop)]
#[entry]
fn main() -> ! {
    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();
    let gpioe = dp.GPIOE.split();

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

    // Configure SPI peripheral
    let mut display = TM1638::new(
        gpioc.pc8,
        gpioc.pc9,
        gpioc.pc10,
    );

    display.write_display(&[0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,]);
    display.write_command(0x8b);

    // Toggle the LED
    let mut shift: u8 = 1;
    loop {
        display.write_display(&[0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,0x3f,]);
        display.write_command(0x8b);

        test_point.reset_all();

        match shift {
            // 0x01 => { set!(test_point, 1)},
            // 0x02 => { set!(test_point, 2)},
            // 0x04 => { set!(test_point, 3)},
            // 0x08 => { set!(test_point, 4)},
            // 0x10 => { set!(test_point, 5)},
            // 0x20 => { set!(test_point, 6)},
            // 0x40 => { set!(test_point, 7)},
            0x80 => { set!(test_point, 8)},
            _ => {}
        }
        shift <<= 1;
        if shift == 0 {shift = 0x01}

        cortex_m::asm::delay(100);
    }}

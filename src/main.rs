




#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::pac;
use crate::hal::pac::TIM2;
use crate::hal::pac::{USART2, DMA1};
use crate::hal::prelude::*;
use crate::hal::timer::Counter;
use crate::hal::serial::{config::Config, Serial};
use crate::hal::dma;
use crate::hal::dma::Transfer;
use crate::hal::dma::{PeripheralToMemory, MemoryToPeripheral, Stream5, Stream6, StreamsTuple};
use crate::hal::dma::{Channel5, Channel6, config::DmaConfig};


use ws2812_spi as ws2812;


use rtt_target::rprintln;
use rtt_target::rtt_init_print;

mod test_points;
use test_points::{*};

mod ev_charger;
use ev_charger::*;

mod display;
use display::*;

mod pallet;
use pallet::Colors;

mod light_ports;
use light_ports::*;


type DmaRxTransfer =
        Transfer<Stream5<DMA1>, 0, USART2, PeripheralToMemory, &'static mut [u8;1]>;

type DmaBunny =
    Transfer<Stream5<DMA1>, 0, USART2, PeripheralToMemory, &'static mut [u8; 2]>;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    // Acquire the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Configure the RCC (Reset and Clock Control) peripheral to enable GPIOA
    let rcc = dp.RCC.constrain();
    let clocks: hal::rcc::Clocks = rcc.cfgr.sysclk(48.MHz()).freeze();

    let mut sys_timer:Counter<TIM2, 1000> = dp.TIM2.counter_ms(&clocks);
    sys_timer.start(u32::MAX.millis()).unwrap();

    let gpioa = dp.GPIOA.split();
    // let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    // let gpiod = dp.GPIOD.split();
    // let gpioe = dp.GPIOE.split();

    let tx = gpioa.pa2.into_alternate();
    let rx = gpioa.pa3.into_alternate();
    let de = gpioa.pa4.into_push_pull_output();

    let usart2 = Serial::new(
        dp.USART2,
        (tx, rx),
        Config::default().baudrate(19200.bps()).parity_none().wordlength_8(),
        &clocks,
    )
    .unwrap();

    // Split UART peripheral into transmitter and receiver
    let (mut uart2_tx, mut uart2_rx) = usart2.split();

    // Initialize DMA
    let dma_channels = StreamsTuple::new(dp.DMA1);
    let mut tx_channel = dma_channels.6;
    let mut rx_channel = dma_channels.5;

    // Create buffers for sending and receiving data
    static mut TX_BUFFER: [u8; 8] = [0; 8];
    static mut RX_BUFFER: [u8; 8] = [0; 8];

    let dma_config = DmaConfig::default()
    .transfer_complete_interrupt(true)
    .memory_increment(true);

    rprintln!("data: {:?}", dma_config);

    let mut rx_transfer = Transfer::init_peripheral_to_memory(
        rx_channel,
        uart2_rx,
        unsafe { &mut RX_BUFFER },
        None,
        dma_config,
        );


    rx_transfer.start(|_| {unsafe{rprintln!("data: {:?}", RX_BUFFER);}});


    // Configure PA5 as a digital output
    let mut test_point = TestPoints::new(
        gpioc.pc0, gpioc.pc1, gpioc.pc2, gpioc.pc3, gpioc.pc4, gpioc.pc5, gpioc.pc6, gpioc.pc7,
    );
    test_point.reset_all();

    // Configure SPI peripheral
    let mut display = TM1638::new(gpioc.pc8, gpioc.pc9, gpioc.pc10,);

    display.initialize(7);
    display.set_brightness(7);

    let mut buffer = [0u8; (LED_NUM * 12) + 30];
    let mut lights = LightPorts::new(gpioa.pa5, gpioa.pa7, dp.SPI1, &mut buffer, &clocks, &sys_timer);

    let mut chargers = [
        EVCharger::new(1, 0),
        EVCharger::new(2, 1),
        EVCharger::new(3, 2),
        EVCharger::new(4, 3),
    ];


    loop {
        let mut updated = false;
        for chrg in &mut chargers {
            if chrg.refresh_ui(&mut display, &mut lights) {
                updated = true;
            }
        }

        lights.refresh( updated);

        test_point.reset_all();
        set!(test_point, 6);

        let key_event = display.get_key_events();
        match key_event {
            Some(ev) => {
                for chrg in &mut chargers {
                    chrg.on_key_event(&ev);
                }
            },
            _ => {}
        };

        // cortex_m::asm::delay(1_000_000);
        // this si a bit mickey mouse but it hunts for now
        let timeout = sys_timer.now() + 100.millis();
        while sys_timer.now() < timeout {

        }

        unsafe{rprintln!("beep: {:?}", RX_BUFFER)};

        // let tics = sys_timer.now();
        // let cur = clocks.sysclk();
        // rprintln!("clk: {:?} {:?}", cur, tics);

    }}

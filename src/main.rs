




#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use crate::hal::pac;
use crate::hal::pac::TIM2;
use crate::hal::pac::USART2;
use crate::hal::prelude::*;
use crate::hal::timer::Counter;
use crate::hal::serial::{config::Config, Serial};
use crate::hal::dma::Transfer;
use crate::hal::dma::StreamsTuple;
use crate::hal::dma::config::DmaConfig;


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

fn calculate_crc16(data: &[u8]) -> u16{

    let mut crc: u16 = 0xFFFF;  // Initial CRC value
    let poly: u16 = 0xA001;  // CRC-16 polynomial

    for &int_val in data {
        crc ^= int_val as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ poly;
            } else {
                crc >>= 1;
            }
        }
    }

    crc

}

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
    let mut de = gpioa.pa4.into_push_pull_output();

    de.set_high();

    let ser_config = Config::default()
    .baudrate(19200.bps())
    .parity_none()
    .wordlength_8()
    .dma(hal::serial::config::DmaConfig::TxRx);

    rprintln!("ser config: {:?}", ser_config);

    let usart2 = Serial::new(
        dp.USART2,
        (tx, rx),
        ser_config,
        &clocks,
    )
    .unwrap();

    // Split UART peripheral into transmitter and receiver
    let (mut uart2_tx, uart2_rx) = usart2.split();

    stm32f4xx_hal::prelude::_embedded_hal_serial_nb_Write::write(&mut uart2_tx, 0x11)
    .unwrap();

    // Initialize DMA
    let dma_channels = StreamsTuple::new(dp.DMA1);
    let _tx_channel = dma_channels.6;
    let rx_channel = dma_channels.5;

    // Create buffers for sending and receiving data
    const BUF_LEN: usize = 20;
    static mut _TX_BUFFER: [u8; 8] = [0; 8];
    static mut RX_BUFFER: [u8; BUF_LEN] = [0; BUF_LEN];

    let dma_config = DmaConfig::default()
    .transfer_complete_interrupt(true)
    .memory_increment(true);

    rprintln!("dma config: {:?}", dma_config);

    let mut rx_transfer = Transfer::init_peripheral_to_memory(
        rx_channel,
        uart2_rx,
        unsafe { &mut RX_BUFFER },
        None,
        dma_config,
        );

    rx_transfer.start(|p: &mut hal::uart::Rx<USART2>| rprintln!("data: {:?}", p.is_rx_not_empty()));


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

    let mut last_xfs = BUF_LEN as u16;
    let mut last_rcv_to: Option<fugit::Instant<u32, 1, 1000>> = None;

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
        let timeout: fugit::Instant<u32, 1, 1000> = sys_timer.now() + 1.millis();
        while sys_timer.now() < timeout {

        }

        de.set_low();

        let xfrs = rx_transfer.number_of_transfers();
        if last_xfs != xfrs {
            last_rcv_to = Some(sys_timer.now() + 3.millis());
            last_xfs = xfrs;
        }

        match last_rcv_to {
            Some(tout) if sys_timer.now() >= tout  => {
                let tx_size = BUF_LEN - xfrs  as usize;
                let msg = unsafe{&RX_BUFFER[0..tx_size]};
                let crc = calculate_crc16(msg);
                rprintln!("beep: {:?} {:?}", crc, &msg[..tx_size-2]);
                last_rcv_to = None;
                last_xfs = BUF_LEN as u16;
                unsafe{RX_BUFFER = [0; BUF_LEN]};
                rx_transfer.next_transfer(unsafe{&mut RX_BUFFER});
                rx_transfer.start(|_| ());
            },
            _ => {}
        }

        // let tics = sys_timer.now();
        // let cur = clocks.sysclk();
        // rprintln!("clk: {:?} {:?}", cur, tics);

    }}

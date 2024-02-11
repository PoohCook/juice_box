use stm32f4xx_hal::gpio::*;

pub const DISPLAY_DELAY: u32 = 3;
pub const DIGITS: [u8; 12] = [0x3f, 0x30, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x67, 0x00, 0x40];

pub struct TM1638 {
    stb: Pin<'C', 8, Output>,
    clk: Pin<'C', 9, Output>,
    dio: DynamicPin<'C', 10>,
    disp_buffer: [u8; 8],
    brightness: u8
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
            dio: pc10.into_dynamic(),
            disp_buffer: [0,0,0,0,0,0,0,0],
            brightness: 7
        };

        me.stb.set_high();
        me.clk.set_high();
        me.dio.make_pull_up_input();

        me
    }

    fn write_byte(&mut self, data: u8){

        for i in 0..8 {
            self.clk.set_low();
            let mask: u8 = 0x01 << i;
            if data & mask == 0 {
               self.dio.set_low().unwrap();
            } else {
                self.dio.set_high().unwrap();
            }

            cortex_m::asm::delay(DISPLAY_DELAY);
            self.clk.set_high();
            cortex_m::asm::delay(DISPLAY_DELAY);
        }

    }

    fn write_display(&mut self, data: &[u8; 8]) {
        self.dio.make_push_pull_output();

        self.stb.set_low();
        cortex_m::asm::delay(DISPLAY_DELAY);
        self.write_byte(0x40);

        for d in data {
            self.write_byte(*d);
            self.write_byte(0);
        }

        self.stb.set_high();
        cortex_m::asm::delay(DISPLAY_DELAY);
    }

    fn write_command(&mut self, command: u8){
        self.dio.make_push_pull_output();

        self.stb.set_low();
        cortex_m::asm::delay(DISPLAY_DELAY);
        self.write_byte(command);
        self.stb.set_high();
        cortex_m::asm::delay(DISPLAY_DELAY);

    }

    fn write_display_command (&mut self){
        self.write_command(0x88 | self.brightness);
    }

    fn update_buffer(&mut self, bank: u8, disp: &[u8; 2]){

        match bank {
            1 => {
                self.disp_buffer[0..0 + disp.len()].copy_from_slice(disp);
            },
            2 => {
                self.disp_buffer[2..2 + disp.len()].copy_from_slice(disp);
            },
            3 => {
                self.disp_buffer[4..4 + disp.len()].copy_from_slice(disp);
            },
            4 => {
                self.disp_buffer[6..6 + disp.len()].copy_from_slice(disp);
            },
            _ => {}
        }
    }

    pub fn initialize(&mut self, brightness: u8) {
        self.brightness = brightness & 0x07;
        self.write_command(0xc0);
        let data = self.disp_buffer;
        self.write_display(&data);
        self.write_display_command();

    }

    pub fn set_brightness(&mut self, brightness: u8) {
        self.brightness = brightness & 0x07;
        self.write_display_command();

    }


    pub fn display_num (&mut self, bank: u8, number: u8){

        let mut disp: [u8; 2] = [DIGITS[11], DIGITS[11]];
        if number < 100 {
            let mut tens = number / 10;
            let ones = number % 10;

            if tens == 0 {tens = 10};

            disp = [DIGITS[tens as usize], DIGITS[ones as usize]];
        }

        self.update_buffer(bank, &disp);

        self.write_command(0xc0);
        let data = self.disp_buffer;
        self.write_display(&data);
        self.write_display_command();
    }



}

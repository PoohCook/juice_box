use stm32f4xx_hal::gpio::*;

const DISPLAY_DELAY: u32 = 3;
const DIGITS: [u8; 12] = [0x3f, 0x30, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x67, 0x00, 0x40];

pub enum KeyEvent {
    KeyDown{key: u8},
    KeyUp{key: u8},
    KeyErr,
}


pub struct TM1638 {
    stb: Pin<'C', 8, Output>,
    clk: Pin<'C', 9, Output>,
    dio: DynamicPin<'C', 10>,
    disp_buffer: [u8; 8],
    brightness: u8,
    key_status: u8,
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
            brightness: 7,
            key_status: 0,
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

    fn read_byte(&mut self) -> u8{

        let mut data: u8 = 0;

        for i in 0..8 {
            self.clk.set_low();
            cortex_m::asm::delay(DISPLAY_DELAY);
            self.clk.set_high();
            let mask: u8 = 0x01 << i;
            if self.dio.is_high().unwrap_or_default() == true {
                data |= mask;
            }
            cortex_m::asm::delay(DISPLAY_DELAY);
        }

        data
    }

    fn read_key_bytes(&mut self) -> [u8; 4]{

        let mut data: [u8; 4] = [0,0,0,0];
        for d  in &mut data {
            *d = self.read_byte();
        };

        data

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
            0 => {
                self.disp_buffer[0..0 + disp.len()].copy_from_slice(disp);
            },
            1 => {
                self.disp_buffer[2..2 + disp.len()].copy_from_slice(disp);
            },
            2 => {
                self.disp_buffer[4..4 + disp.len()].copy_from_slice(disp);
            },
            3 => {
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

    pub fn read_buttons(&mut self) -> u8{

        self.dio.make_push_pull_output();

        self.stb.set_low();
        cortex_m::asm::delay(DISPLAY_DELAY);
        self.write_byte(0x42);

        self.dio.make_floating_input();
        cortex_m::asm::delay(10);

        let data_bytes = self.read_key_bytes();
        let mut keys: u8 = 0;

        self.dio.make_push_pull_output();
        self.stb.set_high();
        cortex_m::asm::delay(DISPLAY_DELAY);


        for i in 0..4{
            if data_bytes[i] & 0x04 != 0 {
                keys |= 0x01 << (i * 2);
            }
            if data_bytes[i] & 0x40 != 0 {
                keys |= 0x01 << ((i * 2) + 1);
            }
        }

        keys

    }

    pub fn get_key_events(& mut self) -> Option<KeyEvent> {

        let key_data = self.read_buttons();
        if self.key_status == key_data{
            return None;
        }

        let diff = self.key_status ^ key_data;
        if count_bits_high(diff) > 1 {
            return Some(KeyEvent::KeyErr);
        }

        self.key_status = key_data;

        if diff & key_data != 0 {
            return Some(KeyEvent::KeyDown { key: find_first_high(diff) });
        }

        Some(KeyEvent::KeyUp { key: find_first_high(diff) })
    }

}

fn count_bits_high(value: u8) -> u8 {
    let mut count = 0;
    for i in 0..8 {
        if value & (1 << i) != 0 {
            count += 1;
        }
    }
    count
}

fn find_first_high(key_data: u8) -> u8 {

    for i in 0..8 {
        if key_data & (1 << i) != 0 {
            return i+1;
        }
    };

    0
}


use rtt_target::rprintln;


#[derive(PartialEq, Debug)]
pub enum Reference {
    Size(u8),
    Address(u16)
}

#[derive(PartialEq, Debug)]
pub struct ModbusFrame {
    pub unit_id: u8,
    pub command: u8,
    pub refers: Reference,
    pub value: u16
}

impl ModbusFrame {

    pub fn new(unit_id: u8,
        command: u8,
        refers: Reference,
        value: u16) -> Self {
            Self {
                unit_id,
                command,
                refers,
                value
            }
    }

    pub fn encode(&self, buffer: &mut [u8]) -> usize {

        let mut len = 0;
        buffer[len] = self.unit_id;
        len += 1;
        buffer[len] = self.command;
        len += 1;
        match self.refers {
            Reference::Size(s) => {
                buffer[len] = s;
                len += 1;
            },
            Reference::Address(adr) => {
                let tmp = Self::u16_to_u8_array(adr);
                buffer[len..len+2].copy_from_slice(&tmp);
                len += 2;
            },
        }

        let tmp = Self::u16_to_u8_array(self.value);
        buffer[len..len+2].copy_from_slice(&tmp);
        len += 2;

        let crc = Self::calculate_crc16(&buffer[..len]);
        let mut crc = Self::u16_to_u8_array(crc);
        crc.reverse();
        buffer[len..len+2].copy_from_slice(&crc);
        len += 2;

        rprintln!("encoded: {:?}", &buffer[..len]);

        len
    }

    pub fn decode(buffer: &[u8]) -> Result<Self, &str> {
        rprintln!("decoded: {:?}", buffer);

        let crc = Self::calculate_crc16(buffer);
        if crc != 0 { return Err("bad crc")};

        Ok(Self {
            unit_id: buffer[0],
            command: buffer[1],
            refers: Reference::Address(Self::u8_array_to_u16(&buffer[2..4].try_into().unwrap())),
            value: Self::u8_array_to_u16(&buffer[4..6].try_into().unwrap())
        })
    }

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

    fn u16_to_u8_array(value: u16) -> [u8; 2] {
        let byte1 = ((value >> 8) & 0xFF) as u8;
        let byte0 = (value & 0xFF) as u8;
        [byte1, byte0]
    }

    fn u8_array_to_u16(arr: &[u8; 2]) ->  u16{
        let mut value: u16 = 0;
        value += (arr[0] as u16) << 8;
        value += arr[1] as u16;

        value
    }
}

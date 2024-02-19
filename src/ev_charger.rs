use crate::{KeyEvent, LightPorts, TM1638};
use smart_leds::RGB8;
use crate::COLORS;

pub struct EVCharger {
    unit_id: u8,
    ui_bank: u8,
    status: CStatus,
    update: bool,
    service_key: u8,
    aux_key: u8,
    key_colors: [RGB8; 2],
    bank_color: RGB8,
}

impl EVCharger {
    pub fn new(unit_id: u8, ui_bank: u8) -> Self {
        Self {
            unit_id,
            ui_bank,
            status: CStatus::WAIT,
            update: true,
            service_key: (ui_bank * 2) + 1,
            aux_key: (ui_bank * 2) + 2,
            key_colors: [RGB8::default(), RGB8::default()],
            bank_color: COLORS[3],
        }
    }

    pub fn on_key_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::KeyDown { key } if *key == self.service_key => {
                self.key_colors[0] = COLORS[6];
                self.update = true;
            },
            KeyEvent::KeyUp { key } if *key == self.service_key => {
                self.key_colors[0] = COLORS[0];
                self.update = true;
            },
            KeyEvent::KeyDown { key } if *key == self.aux_key => {
                self.unit_id += 1;
                self.key_colors[1] = COLORS[5];
                self.update = true;
            },
            KeyEvent::KeyUp { key } if *key == self.aux_key => {
                self.key_colors[1] = COLORS[0];
                self.update = true;
            },
            _ => {}
        }
    }

    pub fn refresh_ui(&mut self, display: &mut TM1638, light_ports: &mut LightPorts) {
        if self.update{
            display.display_num(self.ui_bank, self.unit_id);
            light_ports.set_bar(self.ui_bank, self.bank_color).unwrap();
            for i in 0..2{
                light_ports.set_button(self.ui_bank, i, self.key_colors[i]).unwrap();
            }
            self.update = false;
        }

    }
}

pub enum CStatus{
    WAIT,
    STANDBY,
    CONNECT,
    CHARGE,
    OUTAGE,
    ABNORMAL,

}

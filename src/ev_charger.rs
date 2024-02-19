use crate::{KeyEvent, LightPorts, TM1638};
use smart_leds::RGB8;
use crate::Colors;

pub struct EVCharger {
    unit_id: u8,
    ui_bank: u8,
    status: CStatus,
    update: bool,
    service_key: u8,
    aux_key: u8,
    key_colors: [RGB8; 2],
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
        }
    }

    pub fn on_key_event(&mut self, event: &KeyEvent) {
        match event {
            KeyEvent::KeyDown { key } if *key == self.service_key => {
                self.advance_state();
            },
            KeyEvent::KeyUp { key } if *key == self.service_key => {
                self.update = true;
            },
            KeyEvent::KeyDown { key } if *key == self.aux_key => {
                self.unit_id += 1;
                self.key_colors[1] = Colors::Blue.as_rgb();
                self.update = true;
            },
            KeyEvent::KeyUp { key } if *key == self.aux_key => {
                self.key_colors[1] = Colors::Black.as_rgb();
                self.update = true;
            },
            _ => {}
        }
    }

    pub fn advance_state(&mut self) {
        match self.status {
            CStatus::WAIT => {
                self.status = CStatus::STANDBY;
                self.update = true;
            },
            CStatus::STANDBY => {
                self.status = CStatus::CONNECT;
                self.update = true;
            },
            CStatus::CONNECT => {
                self.status = CStatus::CHARGE;
                self.update = true;
            },
            CStatus::CHARGE => {
                self.status = CStatus::WAIT;
                self.update = true;
            },
            _ => {}
        }
    }

    fn update_led_status(&self, light_ports: &mut LightPorts){
        match self.status {
            CStatus::WAIT => {
                light_ports.set_bar(self.ui_bank, Colors::Green.as_rgb(), false).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Green.as_rgb(), true).unwrap();
            },
            CStatus::STANDBY => {
                light_ports.set_bar(self.ui_bank, Colors::Yellow.as_rgb(), true).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Yellow.as_rgb(), true).unwrap();
            },
            CStatus::CONNECT => {
                light_ports.set_bar(self.ui_bank, Colors::Orange.as_rgb(), true).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Orange.as_rgb(), true).unwrap();
           },
            CStatus::CHARGE => {
                light_ports.set_bar(self.ui_bank, Colors::Orange.as_rgb(), false).unwrap();
                light_ports.set_button(self.ui_bank, 0, Colors::Orange.as_rgb(), false).unwrap();
            },
            _ => {}
        }

    }

    pub fn refresh_ui(&mut self, display: &mut TM1638, light_ports: &mut LightPorts) -> bool {
        if self.update{
            display.display_num(self.ui_bank, self.unit_id);

            self.update_led_status(light_ports);

            light_ports.set_button(self.ui_bank, 1, self.key_colors[1], false).unwrap();

            self.update = false;

            return true;
        }

        false

    }
}

#[allow(dead_code)]
pub enum CStatus{
    WAIT,
    STANDBY,
    CONNECT,
    CHARGE,
    OUTAGE,
    ABNORMAL,

}

use stm32f4xx_hal::{gpio::*};

pub struct TestPoints {
    pub tp1: Pin<'C', 0, Output<PushPull>>,
    pub tp2: Pin<'C', 1, Output<PushPull>>,
    pub tp3: Pin<'C', 2, Output<PushPull>>,
    pub tp4: Pin<'C', 3, Output<PushPull>>,
    pub tp5: Pin<'C', 4, Output<PushPull>>,
    pub tp6: Pin<'C', 5, Output<PushPull>>,
    pub tp7: Pin<'C', 6, Output<PushPull>>,
    pub tp8: Pin<'C', 7, Output<PushPull>>,
}
impl TestPoints {
    pub fn new(parts: gpioc::Parts) -> Self {
        TestPoints{
            tp1: parts.pc0.into_push_pull_output(),
            tp2: parts.pc1.into_push_pull_output(),
            tp3: parts.pc2.into_push_pull_output(),
            tp4: parts.pc3.into_push_pull_output(),
            tp5: parts.pc4.into_push_pull_output(),
            tp6: parts.pc5.into_push_pull_output(),
            tp7: parts.pc6.into_push_pull_output(),
            tp8: parts.pc7.into_push_pull_output(),
        }
    }

    pub fn reset_all(&mut self){
        self.tp1.set_high();
        self.tp2.set_high();
        self.tp3.set_high();
        self.tp4.set_high();
        self.tp5.set_high();
        self.tp6.set_high();
        self.tp7.set_high();
        self.tp8.set_high();

    }

}


#[macro_export]
macro_rules! reset {
    ($test_point:expr, $tp_num:expr) => {
        match $tp_num {
            1 => $test_point.tp1.set_high(),
            2 => $test_point.tp2.set_high(),
            3 => $test_point.tp3.set_high(),
            4 => $test_point.tp4.set_high(),
            5 => $test_point.tp5.set_high(),
            6 => $test_point.tp6.set_high(),
            7 => $test_point.tp7.set_high(),
            8 => $test_point.tp8.set_high(),
            _ => {} // Handle invalid test point number
        }
    };
}

#[macro_export]
macro_rules! set {
    ($test_point:expr, $tp_num:expr) => {
        match $tp_num {
            1 => $test_point.tp1.set_low(),
            2 => $test_point.tp2.set_low(),
            3 => $test_point.tp3.set_low(),
            4 => $test_point.tp4.set_low(),
            5 => $test_point.tp5.set_low(),
            6 => $test_point.tp6.set_low(),
            7 => $test_point.tp7.set_low(),
            8 => $test_point.tp8.set_low(),
            _ => {} // Handle invalid test point number
        }
    };
}

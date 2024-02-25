use stm32f4xx_hal::gpio::*;

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

#[allow(dead_code)]
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
    /// Creates a new TestPoints structure.
    ///
    /// Structure allows setting and reseting of TestPoint IO
    /// # Arguments
    ///
    /// * `pc0` - GPIO for TP1.
    /// * `pc1` - GPIO for TP2.
    /// * `pc2` - GPIO for TP3.
    /// * `pc3` - GPIO for TP4.
    /// * `pc4` - GPIO for TP5.
    /// * `pc5` - GPIO for TP6.
    /// * `pc6` - GPIO for TP7.
    /// * `pc7` - GPIO for TP8.
    ///
    /// # Returns
    ///
    /// The TestPoints Instance
    ///
    pub fn new(
        pc0: Pin<'C', 0>,
        pc1: Pin<'C', 1>,
        pc2: Pin<'C', 2>,
        pc3: Pin<'C', 3>,
        pc4: Pin<'C', 4>,
        pc5: Pin<'C', 5>,
        pc6: Pin<'C', 6>,
        pc7: Pin<'C', 7>,

    ) -> Self {
        TestPoints{
            tp1: pc0.into_push_pull_output(),
            tp2: pc1.into_push_pull_output(),
            tp3: pc2.into_push_pull_output(),
            tp4: pc3.into_push_pull_output(),
            tp5: pc4.into_push_pull_output(),
            tp6: pc5.into_push_pull_output(),
            tp7: pc6.into_push_pull_output(),
            tp8: pc7.into_push_pull_output(),
        }
    }

    /// Reset all TPs to high value (LED Off)
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

    #[allow(dead_code)]
    /// Write all test points using a bit mask
    ///
    ///  mask value of 0x01 denotes TP set and all other TPs as reset
    pub fn write_value(&mut self, val: u8){
        self.reset_all();

        for i in 0..8 {
            let mask = 0x01 << i;
            if val & mask != 0 {
                set!(self, i+1)
            }
        }
    }

}

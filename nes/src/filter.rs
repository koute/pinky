use crate::float::F32;

// This is a simple antialiasing IIR filter for 352.8 Khz (8 * 44.1 Khz) sampling rate.
//
// It should be good enough to get rid of the majority of audible aliasing.
// This particular filter isn't exactly flat in the passband region, but
// it isn't really noticeable, and considering how cheap it is it's probably
// worth it.
pub struct Filter {
    delay_00: F32,
    delay_01: F32,
    delay_02: F32,
    delay_03: F32,
    delay_04: F32,
    delay_05: F32,
}

impl Filter {
    pub const fn new() -> Filter {
        Filter {
            delay_00: f32!(0.0),
            delay_01: f32!(0.0),
            delay_02: f32!(0.0),
            delay_03: f32!(0.0),
            delay_04: f32!(0.0),
            delay_05: f32!(0.0),
        }
    }

    pub fn apply( &mut self, input: F32 ) -> F32 {
        let v17 = f32!(0.88915976376199868) * self.delay_05;
        let v14 = f32!(-1.8046931203033707) * self.delay_02;
        let v22 = f32!(1.0862126905669063) * self.delay_04;
        let v21 = f32!(-2.0) * self.delay_01;
        let v16 = f32!(0.97475300535003617) * self.delay_04;
        let v15 = f32!(0.80752903209625071) * self.delay_03;
        let v23 = f32!(0.022615049608677419) * input;
        let v12 = f32!(-1.7848029270188865) * self.delay_00;
        let v04 = -v12 + v23;
        let v07 = v04 - v15;
        let v18 = f32!(0.04410421960695305) * v07;
        let v13 = f32!(-1.8500161310426058) * self.delay_01;
        let v05 = -v13 + v18;
        let v08 = v05 - v16;
        let v19 = f32!(1.0876279697671658) * v08;
        let v10 = v19 + v21;
        let v11 = v10 + v22;
        let v06 = v11 - v14;
        let v09 = v06 - v17;
        let v20 = f32!(1.3176796030365203) * v09;
        let output = v20;
        self.delay_05 = self.delay_02;
        self.delay_04 = self.delay_01;
        self.delay_03 = self.delay_00;
        self.delay_02 = v09;
        self.delay_01 = v08;
        self.delay_00 = v07;

        output
    }
}

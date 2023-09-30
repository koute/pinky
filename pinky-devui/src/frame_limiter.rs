use std::thread;
use std::time::Duration;

fn get_time() -> u64 {
    clock_ticks::precise_time_ns() / 1000
}

fn delay( microseconds: u64 ) {
    let s = microseconds / 1000000;
    let n = ((microseconds - (s * 1000000)) * 1000) as u32;
    thread::sleep( Duration::new( s, n ) );
}

fn add_percent( value: u64, percent: u64 ) -> u64 {
    (value * 100 + value * percent) / 100
}

pub struct FrameLimiter {
    target_fps: u64,
    time_per_frame: u64,
    begin_timestamp: u64,
    last_frame_timestamp: u64,
    average_frametime: u64,
    average_delaytime: u64,

    frame_counter: u64,
    frame_counter_timestamp: u64,
    fps: u64
}

impl FrameLimiter {
    pub fn new( target_fps: u64 ) -> FrameLimiter {
        let mut frame_limiter = FrameLimiter {
            target_fps: 0,
            time_per_frame: 0,
            begin_timestamp: 0,
            last_frame_timestamp: 0,
            average_frametime: 0,
            average_delaytime: 0,
            frame_counter: 0,
            frame_counter_timestamp: 0,
            fps: 0
        };

        frame_limiter.reset( target_fps );
        frame_limiter
    }

    fn time_left_until_deadline( &self, current: u64 ) -> u64 {
        let target = self.last_frame_timestamp + self.time_per_frame;
        if current > target {
            0
        } else {
            target - current
        }
    }

    #[allow(dead_code)]
    pub fn target_fps( &self ) -> u64 {
        self.target_fps
    }

    #[allow(dead_code)]
    pub fn fps( &self ) -> u64 {
        self.fps
    }

    pub fn reset( &mut self, target_fps: u64 ) {
        self.target_fps = target_fps;
        self.time_per_frame = 1000000 / self.target_fps;
        self.last_frame_timestamp = 0;
        self.average_frametime = self.time_per_frame / 2;
        self.average_delaytime = 1000;
        self.frame_counter = 0;
        self.frame_counter_timestamp = get_time();
        self.fps = 0;
    }

    pub fn begin( &mut self ) -> bool {
        self.begin_timestamp = get_time();
        let time_left = self.time_left_until_deadline( self.begin_timestamp );
        if time_left > add_percent( self.average_frametime, 20 ) {
            delay( 1000 );

            let elapsed = get_time() - self.begin_timestamp;
            self.average_delaytime = (self.average_delaytime + elapsed) / 2;

            if self.average_frametime < self.average_delaytime {
                self.average_frametime = self.average_delaytime;
            }

            false
        } else {
            true
        }
    }

    pub fn end( &mut self ) -> Option< u64 > {
        let current = get_time();
        let elapsed = current - self.begin_timestamp;

        if elapsed < self.average_frametime {
            self.average_frametime = (self.average_frametime * 2 + elapsed) / 3;
        } else {
            self.average_frametime = (self.average_frametime + elapsed * 2) / 3;
        }

        if self.average_frametime < 1000 {
            self.average_frametime = 1000;
        }

        if current - self.last_frame_timestamp > add_percent( self.time_per_frame, 50 ) {
            self.last_frame_timestamp = current;
        } else {
            self.last_frame_timestamp += self.time_per_frame;
        }

        self.frame_counter += 1;
        let elapsed_since_last_fps_update = current - self.frame_counter_timestamp;
        if elapsed_since_last_fps_update >= 1000 * 1000 {
            self.fps = (self.frame_counter * 1000 * 1000) / elapsed_since_last_fps_update;
            self.frame_counter_timestamp = current;
            self.frame_counter = 0;
            Some( self.fps )
        } else {
            None
        }
    }
}

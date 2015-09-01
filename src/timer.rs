//! Provides the Timer struct, which keeps track of the current playback speed and what index we are at.

use std::cmp::Ordering;

/// Keeps track of the current playback speed and what index we are at.
pub struct Timer {
    dts : Vec<f32>, // possible dt values
    dti : isize, // which of dts we're talking about. 0 is stop, 1 => dts[0], -1 => -dts[0]
    len : Option<usize>, // length of what we're iterating over
    /// Current index, as float; keeps track of partials
    pub t : f32,
    /// Time to pause before re-looping. None means "don't loop".
    pub loop_pause : Option<f32>,
    /// Maximum rate
    pub fps : f32,
}

impl Timer {
    /// Make a new timer
    pub fn new(dts : Vec<f32>, len : Option<usize>) -> Timer {
        let mut new_dts = if dts.is_empty() {
            vec!(1f32)
        } else {
            dts
        };
        new_dts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        Timer {
            dts: new_dts,
            dti: 1,
            len: len,
            t : 0.0,
            loop_pause : None,
            fps : 1.0,
        }
    }
    
    /// set speed to at least a given value. Direction is taken into account.
    /// Returns the new speed.
    pub fn at_least(&mut self, new_speed : f32) -> f32 {
        let signum = new_speed.signum();
        let abs_speed = new_speed * signum;
        for new_ix in 0isize..((self.dts.len()+1) as isize) {
            let speed = if new_ix == 0 {0.} else {self.dts[(new_ix-1) as usize]};
            self.dti = new_ix * (signum as isize);
            if speed >= abs_speed {
                break;
            }
        }
        self.get_dt()
    }

    /// Switch forwards vs. backwards. If stopped, it stays stopped.
    pub fn switch_direction(&mut self) {
        self.dti = -self.dti;
    }

    /// Increment faster
    pub fn faster(&mut self) {
        self.dti = match self.dti {
            0 => 1,
            i if i >= self.dts.len() as isize => self.dts.len() as isize,
            i if i <= -(self.dts.len() as isize) => -(self.dts.len() as isize),
            i if i > 0 => i+1,
            i => i-1
        };
    }

    /// Increment slower
    pub fn slower(&mut self) {
        self.dti = match self.dti {
            0 => 0,
            i if i > 0 => i-1,
            i => i+1
        };
    }

    /// Get current dt
    pub fn get_dt(&self) -> f32{
        match self.dti {
            0 => 0.,
            i if i > 0 => self.dts[(i-1) as usize],
            i => -self.dts[(-1-i) as usize]
        }
    }
    
    /// Current time, as f32
    pub fn get_time(&self) -> f32 {
        self.t
    }

    /// Increment the timer, and return current index
    pub fn incr(&mut self) {
        self.t += self.get_dt() / self.fps;
        
        match (self.len, self.loop_pause) {
            (None, _) if self.t < 0. => {
                    self.t = 0.;
                },
            (_, None) if self.t < 0. => {
                    // fixed length, but no loop, but t is negative
                    self.t = 0.;
                },
            (Some(len), Some(pause)) => {
                // We have a fixed length, but we loop after a pause.
                let loop_len = (len as f32) + pause;
                if self.t < 0. {self.t += loop_len;}
                else if self.t > loop_len {self.t -= loop_len;};
            },
            _ => {}
        }
    }
    
    /// Total time before looping
    pub fn total_loop_time(&self) -> Option<f32> {
        match (self.len, self.loop_pause) {
            (None, _) => {
                    None
            },
            (Some(len), None) => {
                Some(len as f32)
            },
            (Some(len), Some(pause)) => {
                Some((len as f32) + pause)
            }
        }
    }
    
    /// Get the current index into the array
    pub fn get_index(&self) -> usize {
        match (self.len, self.loop_pause) {
            (None, _) if self.t < 0. => {
                    0
                },
            (None, _) => self.t as usize,
            (_, None) if self.t < 0. => {
                    // fixed length, but no loop, but t is negative
                    0
                },
            (Some(len), None) => {
                // We have a fixed length, but we don't loop.
                let ix = self.t as usize;
                if ix >= len { len - 1 } else { ix }
            }
            (Some(len), Some(pause)) => {
                // We have a fixed length, but we don't loop.
                let loop_len = (len as f32) + pause;
                 let ix = (self.t % loop_len) as usize;
                 if ix >= len { len - 1 } else { ix }
            }
        }
    }
}

#[cfg(test)]
mod test {
    
    #[test]
    fn timer_dts() {
        let mut t = ::Timer::new(vec!(1.,2.,4.), None);
        assert_eq!(t.get_dt(), 1.);
        t.incr();
        assert_eq!(t.get_index(), 1);
        t.faster();
        assert_eq!(t.get_dt(), 2.);
        t.incr();
        assert_eq!(t.get_index(), 3);
        t.faster();
        assert_eq!(t.get_dt(), 4.);
        t.incr();
        assert_eq!(t.get_index(), 7);
        t.faster();
        assert_eq!(t.get_dt(), 4.);
        t.incr();
        assert_eq!(t.get_index(), 11);
        t.switch_direction();
        assert_eq!(t.get_dt(), -4.);
        t.incr();
        assert_eq!(t.get_index(), 7);
        t.faster();
        assert_eq!(t.get_dt(), -4.);
        t.incr();
        assert_eq!(t.get_index(), 3);
        t.switch_direction();
        assert_eq!(t.get_dt(), 4.);
        t.incr();
        assert_eq!(t.get_index(), 7);
        t.switch_direction();
        assert_eq!(t.get_dt(), -4.);
        t.incr();
        assert_eq!(t.get_index(), 3);
        t.slower();
        assert_eq!(t.get_dt(), -2.);
        t.incr();
        assert_eq!(t.get_index(), 1);
        t.slower();
        assert_eq!(t.get_dt(), -1.);
        t.incr();
        assert_eq!(t.get_index(), 0);
        t.slower();
        assert_eq!(t.get_dt(), 0.);
        t.incr();
        assert_eq!(t.get_index(), 0);
        t.slower();
        assert_eq!(t.get_dt(), 0.);
        t.incr();
        assert_eq!(t.get_index(), 0);
    }
    
    #[test]
    fn timer_pauseloop() {
        let mut t = ::Timer::new(vec!(1.,2.,4.), Some(5));
        t.loop_pause = Some(5.);
        t.fps = 2.;
        t.faster();
        t.faster();
        assert_eq!(t.get_dt(), 4.);
        t.incr();
        assert_eq!(t.get_index(), 2); // t = 2.
        t.incr();
        assert_eq!(t.get_index(), 4); // t = 4.
        t.incr();
        assert_eq!(t.get_index(), 4); // t = 6.
        t.incr();
        assert_eq!(t.get_index(), 4); // t = 8.
        t.incr();
        assert_eq!(t.get_index(), 0); // t = 10.
    }
}
        

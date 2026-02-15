use crate::core::dungeon::{COLNO, ROWNO};
use crate::util::rng::NetHackRng;
use serde::{Deserialize, Serialize};

pub const MAXRECT: usize = 50;
pub const MAXNROFROOMS: usize = 40;
pub const XLIM: usize = 4;
pub const YLIM: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NhRect {
    pub lx: usize,
    pub ly: usize,
    pub hx: usize,
    pub hy: usize,
}

pub struct RectManager {
    pub rects: [NhRect; MAXRECT + 1],
    pub count: usize,
}

impl RectManager {
    pub fn new() -> Self {
        let mut manager = Self {
            rects: [NhRect {
                lx: 0,
                ly: 0,
                hx: 0,
                hy: 0,
            }; MAXRECT + 1],
            count: 0,
        };
        manager.init();
        manager
    }

    ///
    pub fn init(&mut self) {
        self.count = 1;
        self.rects[0] = NhRect {
            lx: 0,
            ly: 0,
            hx: COLNO - 1,
            hy: ROWNO - 1,
        };
    }

    ///
    pub fn get_rect_ind(&self, r: &NhRect) -> i32 {
        for i in 0..self.count {
            let rectp = &self.rects[i];
            if r.lx == rectp.lx && r.ly == rectp.ly && r.hx == rectp.hx && r.hy == rectp.hy {
                return i as i32;
            }
        }
        -1
    }

    ///
    pub fn get_rect(&self, r: &NhRect) -> Option<&NhRect> {
        for i in 0..self.count {
            let rectp = &self.rects[i];
            if r.lx >= rectp.lx && r.ly >= rectp.ly && r.hx <= rectp.hx && r.hy <= rectp.hy {
                return Some(rectp);
            }
        }
        None
    }

    ///
    pub fn rnd_rect(&self, rng: &mut NetHackRng) -> Option<&NhRect> {
        if self.count > 0 {
            Some(&self.rects[rng.rn2(self.count as i32) as usize])
        } else {
            None
        }
    }

    ///
    pub fn intersect(r1: &NhRect, r2: &NhRect, r3: &mut NhRect) -> bool {
        if r2.lx > r1.hx || r2.ly > r1.hy || r2.hx < r1.lx || r2.hy < r1.ly {
            return false;
        }

        r3.lx = r2.lx.max(r1.lx);
        r3.ly = r2.ly.max(r1.ly);
        r3.hx = r2.hx.min(r1.hx);
        r3.hy = r2.hy.min(r1.hy);

        if r3.lx > r3.hx || r3.ly > r3.hy {
            return false;
        }
        true
    }

    ///
    pub fn remove_rect(&mut self, r: &NhRect) {
        let ind = self.get_rect_ind(r);
        if ind >= 0 {
            self.count -= 1;
            let last_idx = self.count;
            self.rects[ind as usize] = self.rects[last_idx];
        }
    }

    ///
    pub fn add_rect(&mut self, r: &NhRect) {
        if self.count >= MAXRECT {
            return;
        }
        if self.get_rect(r).is_some() {
            return;
        }
        self.rects[self.count] = *r;
        self.count += 1;
    }

    ///
    pub fn split_rects(&mut self, r1_p_val: NhRect, r2: &NhRect) {
        self.remove_rect(&r1_p_val);

        let mut i = self.count as i32 - 1;
        while i >= 0 {
            let mut inter = NhRect {
                lx: 0,
                ly: 0,
                hx: 0,
                hy: 0,
            };
            let rect_i = self.rects[i as usize]; // Clone to avoid borrow conflict
            if Self::intersect(&rect_i, r2, &mut inter) {
                self.split_rects(rect_i, &inter);
            }
            i -= 1;
        }

        if (r2.ly as i32 - r1_p_val.ly as i32 - 1)
            > (if r1_p_val.hy < ROWNO - 1 {
                2 * YLIM
            } else {
                YLIM + 1
            } + 4) as i32
        {
            let mut r = r1_p_val;
            r.hy = r2.ly - 2;
            self.add_rect(&r);
        }
        if (r2.lx as i32 - r1_p_val.lx as i32 - 1)
            > (if r1_p_val.hx < COLNO - 1 {
                2 * XLIM
            } else {
                XLIM + 1
            } + 4) as i32
        {
            let mut r = r1_p_val;
            r.hx = r2.lx - 2;
            self.add_rect(&r);
        }
        if (r1_p_val.hy as i32 - r2.hy as i32 - 1)
            > (if r1_p_val.ly > 0 { 2 * YLIM } else { YLIM + 1 } + 4) as i32
        {
            let mut r = r1_p_val;
            r.ly = r2.hy + 2;
            self.add_rect(&r);
        }
        if (r1_p_val.hx as i32 - r2.hx as i32 - 1)
            > (if r1_p_val.lx > 0 { 2 * XLIM } else { XLIM + 1 } + 4) as i32
        {
            let mut r = r1_p_val;
            r.lx = r2.hx + 2;
            self.add_rect(&r);
        }
    }
}

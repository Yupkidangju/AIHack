use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutTier {
    Degraded,
    Standard,
    Roomy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TuiLayout {
    pub root: Rect,
    pub map: Rect,
    pub status: Rect,
    pub inspect: Rect,
    pub log: Rect,
    pub command: Rect,
    pub debug: Option<Rect>,
    pub tier: LayoutTier,
}

impl TuiLayout {
    pub fn validate(self) -> Result<(), String> {
        let rects = [self.map, self.status, self.inspect, self.log, self.command];
        for rect in rects {
            if rect.width == 0 || rect.height == 0 {
                return Err("layout contains zero-sized panel".to_string());
            }
            if rect.x + rect.width > self.root.width || rect.y + rect.height > self.root.height {
                return Err("panel exceeds root bounds".to_string());
            }
        }
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                if overlaps(rects[i], rects[j]) {
                    return Err(format!("panel overlap detected: {i} {j}"));
                }
            }
        }
        if self.map.width < 40 || self.map.height < 20 {
            return Err("map viewport smaller than 40x20".to_string());
        }
        Ok(())
    }
}

pub fn compute_layout(width: u16, height: u16) -> TuiLayout {
    let root = Rect::new(0, 0, width, height);
    if width >= 120 && height >= 36 {
        roomy_layout(root)
    } else if width >= 100 && height >= 32 {
        standard_layout(root)
    } else {
        degraded_layout(root)
    }
}

fn degraded_layout(root: Rect) -> TuiLayout {
    let map = Rect::new(0, 0, 40, 20);
    let status = Rect::new(40, 0, root.width.saturating_sub(40), 6);
    let inspect = Rect::new(40, 6, root.width.saturating_sub(40), 6);
    let log = Rect::new(0, 20, root.width, root.height.saturating_sub(23));
    let command = Rect::new(0, root.height.saturating_sub(3), root.width, 3);
    TuiLayout {
        root,
        map,
        status,
        inspect,
        log,
        command,
        debug: None,
        tier: LayoutTier::Degraded,
    }
}

fn standard_layout(root: Rect) -> TuiLayout {
    let map = Rect::new(0, 0, 60, 24);
    let status = Rect::new(60, 0, root.width.saturating_sub(60), 8);
    let inspect = Rect::new(60, 8, root.width.saturating_sub(60), 8);
    let log = Rect::new(0, 24, root.width, root.height.saturating_sub(27));
    let command = Rect::new(0, root.height.saturating_sub(3), root.width, 3);
    TuiLayout {
        root,
        map,
        status,
        inspect,
        log,
        command,
        debug: None,
        tier: LayoutTier::Standard,
    }
}

fn roomy_layout(root: Rect) -> TuiLayout {
    let map = Rect::new(0, 0, 60, 24);
    let status = Rect::new(60, 0, root.width.saturating_sub(60), 8);
    let inspect = Rect::new(60, 8, root.width.saturating_sub(60), 8);
    let debug = Rect::new(60, 16, root.width.saturating_sub(60), 8);
    let log = Rect::new(0, 24, root.width, root.height.saturating_sub(27));
    let command = Rect::new(0, root.height.saturating_sub(3), root.width, 3);
    TuiLayout {
        root,
        map,
        status,
        inspect,
        log,
        command,
        debug: Some(debug),
        tier: LayoutTier::Roomy,
    }
}

fn overlaps(a: Rect, b: Rect) -> bool {
    let ax2 = a.x + a.width;
    let ay2 = a.y + a.height;
    let bx2 = b.x + b.width;
    let by2 = b.y + b.height;
    a.x < bx2 && ax2 > b.x && a.y < by2 && ay2 > b.y
}

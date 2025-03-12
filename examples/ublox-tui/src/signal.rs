#[derive(Clone, Debug, Default)]
pub struct Signal {
    pub points: Vec<(f64, f64)>,
    pub capacity: usize,
    pub _period: f64,
    pub x_bounds: [f64; 2],
    pub y_bounds: [f64; 2],
}

impl Signal {
    pub const fn new(capacity: usize, period: f64) -> Self {
        Self {
            capacity,
            _period: period,
            points: Vec::new(),
            x_bounds: [-10.0, 10.0],
            y_bounds: [-10.0, 10.0],
        }
    }

    pub fn append(&mut self, value: (f64, f64)) {
        if self.points.len() > self.capacity {
            self.points.drain(0..1);
            self.points.push(value);
            self.x_bounds[0] = self.points[0].0;
        } else {
            self.points.push(value);
            if !self.points.is_empty() {
                self.x_bounds[0] = self.points[0].0;
            }
        }
        self.x_bounds[1] = value.0;
        if value.1 < self.y_bounds[0] {
            self.y_bounds[0] = value.1;
        }
        if value.1 > self.y_bounds[1] {
            self.y_bounds[1] = value.1;
        }

        // Calculate adaptive y bounds
        let scale_factor = 1.2;
        self.y_bounds[1] = self
            .points
            .iter()
            .max_by(|&a, &b| a.1.total_cmp(&b.1))
            .unwrap_or(&(0.0, self.y_bounds[1] * scale_factor))
            .1;
        let scale_factor = 0.8;
        self.y_bounds[0] = self
            .points
            .iter()
            .min_by(|&a, &b| a.1.total_cmp(&b.1))
            .unwrap_or(&(0.0, self.y_bounds[0] * scale_factor))
            .1;
    }

    pub fn current(&self) -> f64 {
        if !self.points.is_empty() {
            self.points[self.points.len() - 1].1
        } else {
            0.0
        }
    }
}

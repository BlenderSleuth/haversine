// Implementation of JSF generator from listing_0066_haversine_generator_main.cpp

pub(crate) struct RandomSeries(u64, u64, u64, u64);

impl RandomSeries {
    pub(crate) fn seed(value: u64) -> RandomSeries {
        let mut series = RandomSeries(0xf1ea5eed, value, value, value);

        for _ in 0..20
        {
            series.random();
        }

        series
    }

    pub(crate) fn random(&mut self) -> u64
    {
        let x = self.0.wrapping_sub(self.1.rotate_left(27));
        self.0 = self.1 ^ self.2.rotate_left(17);
        self.1 = self.2.wrapping_add(self.3);
        self.2 = self.3.wrapping_add(x);
        self.3 = x.wrapping_add(self.0);
        return self.3;
    }

    pub(crate) fn random_in_range(&mut self, min: f64, max: f64) -> f64
    {
        let t = self.random() as f64 / u64::MAX as f64;
        (1.0 - t) * min + t * max
    }

    pub(crate) fn random_degree(&mut self, center: f64, radius: f64, max_allowed: f64) -> f64
    {
        let min_val = (center - radius).max(-max_allowed);
        let max_val = (center + radius).min(max_allowed);
        self.random_in_range(min_val, max_val)
    }
}




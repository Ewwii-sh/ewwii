pub trait IterAverage {
    fn avg(self) -> f32;
}

impl<I: Iterator<Item = f32>> IterAverage for I {
    fn avg(self) -> f32 {
        let mut total = 0f32;
        let mut cnt = 0f32;
        for value in self {
            total += value;
            cnt += 1f32;
        }
        total / cnt
    }
}

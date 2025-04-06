use std::{ops::AddAssign, time::Duration};

#[derive(Default)]
pub struct TimeStepper<const TARGET_HZ: u64> {
    processed_time: Duration,
    pending_time: Duration,
}

pub struct TimeStepInfo {
    pub now: Duration,
    pub delta: Duration,
}

pub struct TimeStepIter<'a, const TARGET_HZ: u64> {
    stepper: &'a mut TimeStepper<TARGET_HZ>,
}

impl<const TARGET_HZ: u64> TimeStepper<TARGET_HZ> {
    const TIMESTEP: Duration = Duration::from_micros(1_000_000 / TARGET_HZ);
}

impl<const TARGET_HZ: u64> AddAssign<Duration> for TimeStepper<TARGET_HZ> {
    fn add_assign(&mut self, delta_time: Duration) {
        self.pending_time += delta_time;
    }
}

impl<'a, const TARGET_HZ: u64> IntoIterator for &'a mut TimeStepper<TARGET_HZ> {
    type IntoIter = TimeStepIter<'a, TARGET_HZ>;
    type Item = <<Self as IntoIterator>::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        TimeStepIter { stepper: self }
    }
}

impl<const TARGET_HZ: u64> Iterator for TimeStepIter<'_, TARGET_HZ> {
    type Item = TimeStepInfo;

    fn next(&mut self) -> Option<Self::Item> {
        (self.stepper.pending_time >= TimeStepper::<TARGET_HZ>::TIMESTEP).then(|| {
            let step_info = TimeStepInfo {
                now: self.stepper.processed_time,
                delta: TimeStepper::<TARGET_HZ>::TIMESTEP,
            };
            self.stepper.pending_time -= TimeStepper::<TARGET_HZ>::TIMESTEP;
            tracing::trace!(
                "Timestep from:{:?}, delta:{:?}, remaining:{:?}",
                step_info.now,
                step_info.delta,
                self.stepper.pending_time,
            );
            self.stepper.processed_time += TimeStepper::<TARGET_HZ>::TIMESTEP;
            step_info
        })
    }
}

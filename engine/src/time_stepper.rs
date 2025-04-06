use std::{ops::AddAssign, time::Duration};

#[derive(Default)]
pub struct TimeStepper<const MIN_PERIOD_US: u64, const STEP_US: u64 = MIN_PERIOD_US> {
    processed_time: Duration,
    pending_time: Duration,
}

pub struct TimeStepInfo {
    pub now: Duration,
    pub delta: Duration,
}

pub struct TimeStepIter<'a, const MIN_PERIOD_US: u64, const STEP_US: u64> {
    stepper: &'a mut TimeStepper<MIN_PERIOD_US, STEP_US>,
}

pub const fn frequency_to_micros(frequency: u64) -> u64 {
    1_000_000 / frequency
}

impl<const MIN_PERIOD_US: u64, const STEP_US: u64> TimeStepper<MIN_PERIOD_US, STEP_US> {
    const MIN_PERIOD: Duration = Duration::from_micros(MIN_PERIOD_US);
    const STEP: Duration = Duration::from_micros(STEP_US);
}

impl<const MIN_PERIOD_US: u64, const STEP_US: u64> AddAssign<Duration>
    for TimeStepper<MIN_PERIOD_US, STEP_US>
{
    fn add_assign(&mut self, delta_time: Duration) {
        self.pending_time += delta_time;
    }
}

impl<'a, const MIN_PERIOD_US: u64, const STEP_US: u64> IntoIterator
    for &'a mut TimeStepper<MIN_PERIOD_US, STEP_US>
{
    type IntoIter = TimeStepIter<'a, MIN_PERIOD_US, STEP_US>;
    type Item = <<Self as IntoIterator>::IntoIter as Iterator>::Item;

    fn into_iter(self) -> Self::IntoIter {
        TimeStepIter { stepper: self }
    }
}

impl<const MIN_PERIOD_US: u64, const STEP_US: u64> Iterator
    for TimeStepIter<'_, MIN_PERIOD_US, STEP_US>
{
    type Item = TimeStepInfo;

    fn next(&mut self) -> Option<Self::Item> {
        (self.stepper.pending_time >= TimeStepper::<MIN_PERIOD_US, STEP_US>::MIN_PERIOD).then(
            || {
                let step_info = TimeStepInfo {
                    now: self.stepper.processed_time,
                    delta: TimeStepper::<MIN_PERIOD_US, STEP_US>::STEP
                        .min(self.stepper.pending_time),
                };
                self.stepper.pending_time -= step_info.delta;
                tracing::trace!(
                    "Timestep from:{:.3}s, delta:{:.3}s, remaining:{:.3}s",
                    step_info.now.as_secs_f32(),
                    step_info.delta.as_secs_f32(),
                    self.stepper.pending_time.as_secs_f32(),
                );
                self.stepper.processed_time += step_info.delta;
                step_info
            },
        )
    }
}

// Copyright (c) 2022 MASSA LABS <info@massa.net>

#[cfg(any(
    test,
    feature = "gas_calibration",
    feature = "benchmarking",
    feature = "testing"
))]
mod mock;

#[cfg(all(not(feature = "gas_calibration"), not(feature = "benchmarking")))]
mod scenarios_mandatories;

#[cfg(all(not(feature = "gas_calibration"), not(feature = "benchmarking")))]
mod tests_active_history;

#[cfg(feature = "testing")]
mod interface;

#[cfg(any(
    feature = "gas_calibration",
    feature = "benchmarking",
    feature = "testing"
))]
pub use mock::get_sample_state;

#[cfg(any(
    feature = "gas_calibration",
    feature = "benchmarking",
    feature = "testing"
))]
pub use mock::get_initials_vesting;

#[cfg(any(test, feature = "testing"))]
mod tests_vesting_manager;

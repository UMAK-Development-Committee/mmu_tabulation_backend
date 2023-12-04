pub mod auth;
pub mod candidate;
pub mod category;
pub mod criteria;
pub mod event;
pub mod judge;
pub mod note;
pub mod score;
pub mod college;
pub mod tests;

pub trait Round {
    fn round_to_two_decimals(&self) -> f64;
}


impl Round for f64 {
    fn round_to_two_decimals(&self) -> f64 {
        (self * 100.0).round() / 100.0
    }
}

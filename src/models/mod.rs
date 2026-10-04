pub mod article;
pub mod comment;
mod user;

pub use user::{User, UserPreview};
pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";

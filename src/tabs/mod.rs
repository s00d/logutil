pub mod base;
pub mod bots;
pub mod detailed;
pub mod errors;
pub mod heatmap;
pub mod overview;
pub mod performance;
pub mod requests;
pub mod security;
pub mod sparkline;

pub use detailed::DetailedTab;
pub use overview::OverviewTab;
pub use requests::RequestsTab;
pub use security::SecurityTab;

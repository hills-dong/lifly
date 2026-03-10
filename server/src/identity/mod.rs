pub mod handlers;
pub mod models;
pub mod repo;
pub mod service;

pub use handlers::routes;
pub use models::{
    CreateDeviceRequest, DeviceResponse, LoginRequest, LoginResponse, UpdateProfileRequest,
    UserProfile,
};

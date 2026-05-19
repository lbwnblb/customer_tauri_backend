pub mod app_data;
pub mod uuid;
pub mod platform;
pub mod constants;
pub mod logger;
pub mod http;
mod feige_resp;

pub use app_data::app_data_dir;
pub use uuid::uuid_no_hyphen;
pub use platform::{is_douyin_platform, is_pinduoduo_platform, get_platform_from_id};
pub use constants::{PLATFORM_DOUYIN, PLATFORM_PINDUODUO, PLATFORM_UNKNOWN};
pub use logger::init_logger;
pub use http::{get, get_json, get_with_timeout, post_json, HttpClient, HttpError, HttpErrorKind, HttpResponse};

pub mod app_data;
pub mod uuid;
pub mod timestamp;
pub mod platform;
pub mod constants;
pub mod logger;
pub mod http;
pub mod feige_resp;
pub mod protobuf;
mod doudian_utils;

pub use app_data::app_data_dir;
pub use uuid::uuid_no_hyphen;
pub use timestamp::timestamp_millis;
pub use platform::{is_douyin_platform, is_pinduoduo_platform, get_platform_from_id};
pub use constants::{PLATFORM_DOUYIN, PLATFORM_PINDUODUO, PLATFORM_UNKNOWN};
pub use logger::init_logger;
pub use http::{get, get_json, get_with_timeout, post_json, HttpClient, HttpError, HttpErrorKind, HttpResponse};
pub use feige_resp::{get_message_by_index_v2_range, get_by_conversation};

use std::path::PathBuf;

pub fn app_data_dir() -> String {
   std::env::var("APPDATA").unwrap_or_else(|_| String::new())
}
pub fn app_data_dir_home_index() -> PathBuf {
    let path = std::env::var("APPDATA").unwrap_or_else(|_| String::new());
    PathBuf::from(path).join("customer_home")
}

pub fn app_data_dir_log() -> PathBuf {
    let path = std::env::var("APPDATA").unwrap_or_else(|_| String::new());
    PathBuf::from(path).join("customer_tauri").join("logs")
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_dir() {
        let result = app_data_dir();
        log::info!("{}", result);
    }
}

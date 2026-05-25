use std::path::PathBuf;

pub fn app_data_dir() -> String {
   std::env::var("APPDATA").unwrap_or_else(|_| String::new())
}
pub fn app_data_dir_home_index() -> PathBuf {
    let path = std::env::var("APPDATA").unwrap_or_else(|_| String::new());
    PathBuf::from(path).join("customer_home")
}


pub fn response_cmd_610_get_conversation_info_list_v2_body()-> String {
    "data/response_cmd_610_get_conversation_info_list_v2_body".to_string()
}
pub fn response_cmd_500_message_type_50002_buy_has_new_message_notify()-> String {
    "data/response_cmd_500_message_type_50002_buy_has_new_message_notify".to_string()
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

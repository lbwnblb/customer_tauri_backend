pub fn app_data_dir() -> String {
   std::env::var("APPDATA").unwrap_or_else(|_| String::new())

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

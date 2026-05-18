pub fn uuid_no_hyphen() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

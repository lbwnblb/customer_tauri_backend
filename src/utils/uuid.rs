pub fn uuid_no_hyphen() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

pub fn uuid_with_hyphen() -> String {
    uuid::Uuid::new_v4().to_string()
}

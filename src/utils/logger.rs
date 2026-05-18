use std::io::Write;

pub fn init_logger() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            let ts = chrono::Utc::now()
                .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                .format("%Y-%m-%d %H:%M:%S");
            writeln!(buf, "{} [{}] {}", ts, record.level(), record.args())
        })
        .init();
}

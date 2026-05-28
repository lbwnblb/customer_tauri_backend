use std::io::Write;

pub fn init_logger() {
    #[cfg(debug_assertions)]
    {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
            .format(|buf, record| {
                let ts = chrono::Utc::now()
                    .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                    .format("%Y-%m-%d %H:%M:%S");
                writeln!(buf, "{} [{}] {}", ts, record.level(), record.args())
            })
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        use crate::utils::app_data::app_data_dir_log;

        let log_dir = app_data_dir_log();
        std::fs::create_dir_all(&log_dir).ok();
        let log_file = log_dir.join(format!(
            "customer_tauri_{}.log",
            chrono::Utc::now()
                .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                .format("%Y-%m-%d")
        ));

        fern::Dispatch::new()
            .format(|out, message, record| {
                let ts = chrono::Utc::now()
                    .with_timezone(&chrono::FixedOffset::east_opt(8 * 3600).unwrap())
                    .format("%Y-%m-%d %H:%M:%S");
                out.finish(format_args!("{} [{}] {}", ts, record.level(), message))
            })
            .level(log::LevelFilter::Info)
            .chain(fern::log_file(log_file).unwrap())
            .apply()
            .ok();
    }
}

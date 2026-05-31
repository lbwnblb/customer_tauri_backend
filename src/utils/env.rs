#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Dev,
    Test,
    Prod,
}

/// 当前运行环境。切换时修改此行即可。
pub const CURRENT_ENV: Env = Env::Prod;

impl Env {
    pub fn base_url(self) -> &'static str {
        match self {
            Env::Dev => "http://127.0.0.1:7788",
            Env::Test => "http://127.0.0.1:7788", // TODO: 填写 test 地址
            Env::Prod => "https://kefu.aiyiyong.com",
        }
    }

    pub fn is_dev(self) -> bool {
        self == Env::Dev
    }
}

/// 快捷函数，返回当前环境的 base_url。
pub fn base_url() -> &'static str {
    CURRENT_ENV.base_url()
}

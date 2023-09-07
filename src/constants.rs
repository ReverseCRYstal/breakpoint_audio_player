/// Reserved
// ⏴⏵⏶⏷⏩⏪⏭⏮⏸⏹⏺■▶★☆☐☑↺↻⟲⟳⬅➡⬆⬇⬈⬉⬊⬋⬌⬍⮨⮩⮪⮫⊗✔⛶
// 🔀🔁🔃
// ☜☝☞☟⛃  ♡

pub mod icon {
    /// Next breakpoint
    pub const NEXT_BRK_PT: &str = "⏩";
    /// Previous breakpoint
    pub const PREV_BRK_PT: &str = "⏪";
    pub const PAUSE: &str = "⏸";
    pub const RESUME: &str = "⏵";
    pub const RESET: &str = "⏹";
    pub const MUTED_VOLUME: &str = "🔇";
    pub const NO_VOLUME: &str = "🔈";
    pub const NORMAL_VOLUME: &str = "🔉";
    pub const FULL_VOLUME: &str = "🔊";
}

pub mod literal {
    pub const EXTENSION_NAME: &str = "bax";
    pub const APP_NAME: &str = "断点音频播放器";
    // pre-alpha alpha beta gamma
    pub const TEST_VERSION: &str = "pre-alpha";
    pub const APP_VERSION: &str = env!("APP_VERSION");
    pub const COMMIT_HASH: &str = env!("GIT_HASH");
    pub const RUST_EDITION: &str = env!("RUST_EDITION");
    pub const BUILD_TOOLCHAIN: &str = env!("BUILD_TOOLCHAIN");
    pub const BUILD_TIME: &str = env!("BUILD_TIME");
}

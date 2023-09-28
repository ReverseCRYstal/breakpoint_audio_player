#[allow(unused)]
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
    /// Followings are deprecated
    pub const MUTED_VOLUME: &str = "🔇";
    pub const NO_VOLUME: &str = "🔈";
    pub const NORMAL_VOLUME: &str = "🔉";
    pub const FULL_VOLUME: &str = "🔊";
}

pub mod literal {
    pub const EXTENSION_NAME: &str = "bax";
    pub const APP_NAME: &str = "断点音频播放器";
    // pre-alpha alpha beta gamma
    pub const TEST_VERSION: &str = "Release";
}

pub mod toasts {
    pub const DUR: std::time::Duration = std::time::Duration::from_secs(10);
}

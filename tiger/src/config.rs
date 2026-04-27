// Mirrors main.h: the global Config struct.
// Held in a OnceLock so submodules can read it; main.rs sets it once at startup.

use std::sync::OnceLock;

pub struct Config {
    pub extract_region_timings_path: String,
    pub ilp_mode: bool,
    pub ilp_minimize_objective: bool,
    pub ilp_timeout_seconds: i32,
    pub time_ilp: bool,
    pub percent_regions: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            extract_region_timings_path: String::new(),
            ilp_mode: false,
            ilp_minimize_objective: true,
            ilp_timeout_seconds: 5 * 60,
            time_ilp: false,
            percent_regions: 100.0,
        }
    }
}

static G_CONFIG: OnceLock<Config> = OnceLock::new();

pub fn set_config(c: Config) {
    G_CONFIG.set(c).ok().expect("g_config already initialized");
}

pub fn g_config() -> &'static Config {
    G_CONFIG.get_or_init(Config::default)
}

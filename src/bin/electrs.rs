use anyhow::Result;
// use env_logger::Env;

fn main() -> Result<()> {
    // env_logger::init_from_env(Env::default().default_filter_or("debug"));
    electrs::run()
}

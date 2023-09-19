#[allow(unused)]
fn test_init_once() {
    use std::sync::OnceLock;
    static ONCE: OnceLock<()> = OnceLock::new();

    ONCE.get_or_init(|| {
        env_logger::init_from_env(
            env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
        )
    });
}

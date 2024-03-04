use env_logger::Env;
use fsync::{App, Config};
use libc::EXIT_FAILURE;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config = Config::from_args().unwrap_or_else(|err| {
        eprintln!("Arguments error: {err}");
        std::process::exit(EXIT_FAILURE);
    });

    let mut app = App::new(config);

    if let Err(err) = app.run() {
        eprintln!("Application error: {err}");
        std::process::exit(EXIT_FAILURE);
    }
}

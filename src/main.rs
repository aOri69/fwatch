use fsync::Config;
use libc::EXIT_FAILURE;

fn main() {
    let _config = Config::from_args().unwrap_or_else(|err| {
        eprintln!("Error parsing arguments: {err}");
        std::process::exit(EXIT_FAILURE);
    });
    // println!("{config}");
}

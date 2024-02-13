use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    SystemTime(std::time::SystemTimeError),
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::time::SystemTimeError> for AppError {
    fn from(value: std::time::SystemTimeError) -> Self {
        Self::SystemTime(value)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AppError::IoError(ref e) => write!(f, "IO: {e}"),
            AppError::SystemTime(ref e) => write!(f, "SystemTime: {e}"),
        }
    }
}

pub struct App {
    source: PathBuf,
    destination: PathBuf,
}

impl App {
    pub fn new(config: crate::Config) -> Self {
        let crate::Config {
            source,
            destination,
        } = config;

        log::info!("source path is set to: {:?}", source);
        log::info!("destination path is set to: {:?}", destination);

        Self {
            source,
            destination,
        }
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        let _ = self.source.read_dir()?;
        let _ = self.destination.read_dir()?;

        self.initial_sync()?;

        if let Err(error) = self.watch(self.source.as_path()) {
            log::error!("Error: {error:?}");
        }

        Ok(())
    }

    fn initial_sync(&mut self) -> Result<(), AppError> {
        log::trace!("Initial scan started: {:?}", self.source.as_path());
        let src_entries = App::collect_dir_entries(self.source.as_path());

        for src_entry in src_entries {
            if src_entry.is_file() {
                let dst_entry = Path::new(&self.destination)
                    .join(src_entry.strip_prefix(self.source.as_path()).unwrap());

                let src_meta = fs::metadata(&src_entry)?;
                let src_last_modified = src_meta.modified()?.elapsed()?.as_secs();

                log::trace!("current source entry: {src_entry:?} - {src_last_modified}");
                log::trace!("current destination entry: {dst_entry:?}");

                match fs::metadata(&dst_entry) {
                    Ok(dst_meta) => {
                        let dst_last_modified = dst_meta.modified()?.elapsed()?.as_secs();

                        if src_last_modified != dst_last_modified {
                            log::warn!("Changes detected in path: {src_entry:?}.\n Last modified: {src_last_modified}");
                        }
                    }
                    Err(err) => match err.kind() {
                        std::io::ErrorKind::NotFound => {
                            fs::copy(src_entry, dst_entry)?;
                        }
                        _ => log::error!("{err}"),
                    },
                }
            }
        }

        Ok(())
    }

    fn collect_dir_entries<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
        walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| match entry {
                Ok(entry) => Some(entry.into_path()),
                Err(err) => {
                    log::warn!("{err}");
                    None
                }
            })
            .collect::<Vec<_>>()
    }

    fn watch<P: AsRef<Path>>(&self, path: P) -> notify::Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        for res in rx {
            match res {
                Ok(event) => log::info!("Change: {event:?}"),
                Err(error) => log::error!("Error: {error:?}"),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{App, Config};
    use log::error;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn non_existing_path() {
        init();

        error!("This record will be captured by `cargo test`");

        let mut app = App::new(Config::build("./test".into(), "./test2".into()));

        assert!(app.run().is_ok());
    }
}

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
            AppError::IoError(ref e) => {
                write!(f, "IO: {e}")
            }
            AppError::SystemTime(ref e) => {
                write!(f, "SystemTime: {e}")
            }
        }
    }
}

pub struct App {
    source: PathBuf,
    destination: PathBuf,
}

impl App {
    pub fn new(config: crate::Config) -> Self {
        let crate::Config { source, destination } = config;

        log::info!("source path is set to: {:?}", source);
        log::info!(
            "destination path is set to: {:?}",
            destination
        );

        Self { source, destination }
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
        log::info!(
            "Initial scan started: {:?}",
            self.source.as_path()
        );
        let src_entries = App::collect_dir_entries(self.source.as_path());

        for src_entry in src_entries {
            if src_entry.is_file() {
                // Sync
                self.sync_by_metadata(src_entry)?;
            }
        }

        log::info!(
            "Initial scan finished: {:?}",
            self.source
        );

        Ok(())
    }

    fn copy<P: AsRef<Path>>(&self, src: P) -> Result<(), AppError> {
        let src = src.as_ref();
        let dst = self.build_dest_path(src)?;
        log::info!("copy: {:?}", dst.file_name().unwrap());
        let _ = fs::copy(src, dst)?;
        Ok(())
    }

    fn remove<P: AsRef<Path>>(&self, src: P) -> Result<(), AppError> {
        let src = src.as_ref();
        let dst = self.build_dest_path(src)?;
        log::info!("remove: {:?}", dst.file_name().unwrap());
        Ok(fs::remove_file(dst)?)
    }

    fn build_dest_path<P: AsRef<Path>>(&self, src: P) -> Result<PathBuf, AppError> {
        // Construct destination path with changing prefixes
        let src = src.as_ref().canonicalize()?;
        let src_prefix = self.source.as_path().canonicalize()?;
        let dst_canoncical = &self.destination.canonicalize()?;
        let dst = Path::new(dst_canoncical).join(src.strip_prefix(src_prefix).unwrap());

        log::debug!(
            "src {}",
            src.to_str().unwrap_or_default()
        );
        log::debug!(
            "dst {}",
            dst.to_str().unwrap_or_default()
        );

        Ok(dst)
    }

    fn sync_by_metadata<P: AsRef<Path>>(&self, src: P) -> Result<(), AppError> {
        let src_meta = fs::metadata(&src)?;
        let src_last_modified = src_meta.modified()?.elapsed()?.as_secs();

        let dst = self.build_dest_path(src.as_ref())?;

        match fs::metadata(&dst) {
            Ok(dst_meta) => {
                let dst_last_modified = dst_meta.modified()?.elapsed()?.as_secs();

                log::debug!(
                    "{} modified: {}",
                    src.as_ref().file_name().unwrap().to_str().unwrap(),
                    src_last_modified
                );
                log::debug!(
                    "{} modified: {}",
                    dst.file_name().unwrap().to_str().unwrap(),
                    dst_last_modified
                );

                if src_last_modified != dst_last_modified {
                    // File found and was modified - need to sync
                    log::info!(
                        "syncing(metadata change): {:?}",
                        dst.file_name().unwrap()
                    );
                    let _ = fs::copy(src, dst)?;
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    // File not found - need to sync
                    log::info!(
                        "syncing(file not present): {:?}",
                        dst.file_name().unwrap()
                    );
                    let _ = fs::copy(src, dst)?;
                }
                _ => log::error!("{err}"),
            },
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
        use notify::EventKind;

        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        log::info!("watch started: {:?}", path.as_ref());

        for res in rx {
            match res {
                Ok(event) => {
                    log::info!("Change: {event:?}");
                    match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) => {
                            event.paths.iter().for_each(|p| {
                                if let Err(e) = self.copy(p) {
                                    log::error!("{e}");
                                }
                            });
                        }
                        EventKind::Remove(_) => event.paths.iter().for_each(|p| {
                            if let Err(e) = self.remove(p) {
                                log::error!("{e}");
                            }
                        }),
                        _ => todo!(),
                    }
                }
                Err(error) => {
                    log::error!("Error: {error:?}")
                }
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

        error!("tests not implemented");

        let mut _app = App::new(Config::build(
            "./test".into(),
            "./test2".into(),
        ));

        // assert!(app.run().is_ok());
        todo!()
    }
}

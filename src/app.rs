//! Main worker module
//! Represented by [App] structure.

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Application error wrapper
#[derive(Debug)]
pub enum AppError {
    /// [Error](std::error::Error) wrapper to represent errors from Input/Output
    IoError(std::io::Error),
    /// [SystemTimeError](std::time::SystemTimeError) wrapper
    SystemTime(std::time::SystemTimeError),
    /// Generic Path error. Mostly represents invalid paths.
    PathErr(String),
    /// [StripPrefixError](std::path::StripPrefixError) wrapper.
    /// Used in ['build_dest_path()'] as error propogation from [std::path::Path::strip_prefix()] function
    StripPrefix(std::path::StripPrefixError),
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

impl From<std::path::StripPrefixError> for AppError {
    fn from(value: std::path::StripPrefixError) -> Self {
        Self::StripPrefix(value)
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            AppError::IoError(ref e) => write!(f, "IO: {e}"),
            AppError::SystemTime(ref e) => write!(f, "SystemTime: {e}"),
            AppError::PathErr(ref e) => write!(f, "Path error: {e}"),
            AppError::StripPrefix(ref e) => write!(f, "Strip Prefix: {e}"),
        }
    }
}

/// Main worker.
///
/// Contains two paths:
/// source and destination as [PathBuf]
pub struct App {
    /// Source path to monitor changes
    source: PathBuf,
    /// Destination path for syncronisation
    destination: PathBuf,
}

impl App {
    /// Application constructor.
    ///
    /// Accepts [Config](crate::Config) as an input.
    pub fn new(config: crate::Config) -> Self {
        let crate::Config { source, destination } = config;

        log::info!("source path is set to: {:?}", source);
        log::info!(
            "destination path is set to: {:?}",
            destination
        );

        Self { source, destination }
    }

    /// Main worker method.
    /// todo!()
    pub fn run(&mut self) -> Result<(), AppError> {
        // Just an error propogation
        let _ = self.source.read_dir()?;
        let _ = self.destination.read_dir()?;
        // Initial scan of source directory
        // with copying everything mismatched
        self.initial_sync()?;
        // Main watch event handler
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

    fn rename<P: AsRef<Path>>(&self, from: P, to: P) -> Result<(), AppError> {
        let new_filename = to.as_ref().file_name().unwrap();
        let old_filename = from.as_ref().file_name().unwrap();
        let destination = self.build_dest_path(to.as_ref())?;

        let from = destination.with_file_name(old_filename);
        let to = destination.with_file_name(new_filename);

        log::info!("renaming:\n{:?}\n{:?}", from, to);

        Ok(fs::rename(from, to)?)
    }

    fn copy<P: AsRef<Path>>(&self, src: P) -> Result<(), AppError> {
        let src = src.as_ref();
        let dst = self.build_dest_path(src)?;
        log::info!("copy: {:?}", dst.file_name().unwrap());

        if src.is_dir() {
            log::debug!("IS DIRECTORY: {src:?}");
            fs::create_dir_all(dst.as_path())?;
            return Ok(());
        }

        match fs::copy(src, dst.as_path()) {
            Ok(_) => Ok(()),
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    fs::create_dir_all(dst.as_path().parent().unwrap())?;
                    fs::copy(src, dst)?;
                    Ok(())
                }
                _ => {
                    log::error!("{err}");
                    Err(err.into())
                }
            },
        }
    }

    fn remove<P: AsRef<Path>>(&self, src: P) -> Result<(), AppError> {
        let src = src.as_ref();
        let dst = self.build_dest_path(src)?;
        log::info!("remove: {:?}", dst.file_name().unwrap());

        // src doesn't exist anymore
        if dst.is_dir() {
            log::debug!("IS DIRECTORY: {src:?}");
            fs::remove_dir(dst.as_path())?;
            return Ok(());
        }

        Ok(fs::remove_file(dst)?)
    }

    fn build_dest_path<P: AsRef<Path>>(&self, from_str: P) -> Result<PathBuf, AppError> {
        let src_str = from_str.as_ref().to_string_lossy().to_string();
        let soruce_prefix = self.source.as_path().to_string_lossy().to_string();
        if let Some(mut offset) = src_str.find(&soruce_prefix) {
            let prefix = match offset {
                0 => self.source.as_path(),
                _ => {
                    offset += soruce_prefix.len();
                    log::debug!(
                        "counted offset for {} == {}:",
                        src_str,
                        offset
                    );
                    Path::new(src_str.get(..offset).ok_or(AppError::PathErr(soruce_prefix.clone()))?)
                }
            };
            let src_stripped = from_str.as_ref().strip_prefix(prefix)?;
            let result = Path::new(self.destination.as_path()).join(src_stripped);

            log::debug!(
                "buildig destination:\nsource path: {}\nstripped to: {:?}\nresult: {:?}",
                &src_str,
                src_stripped,
                result
            );
            return Ok(result);
        }

        Err(AppError::PathErr(soruce_prefix))
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
                    // let _ = fs::copy(src, dst)?;
                    self.copy(src)?;
                }
            }
            Err(err) => match err.kind() {
                std::io::ErrorKind::NotFound => {
                    // File not found - need to sync
                    log::info!(
                        "syncing(file not present): {:?}",
                        dst.file_name().unwrap()
                    );
                    // let _ = fs::copy(src, dst)?;
                    self.copy(src)?;
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
        use notify::event::ModifyKind;
        use notify::event::RenameMode;
        use notify::EventKind;

        let (tx, rx) = std::sync::mpsc::channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        log::info!("watch started: {:?}", path.as_ref());
        // 95 percent of cases there should be only one path
        let mut files_to_rename = Vec::with_capacity(1);

        for res in rx {
            match res {
                Ok(event) => {
                    log::trace!("Change: {event:?}");
                    match event.kind {
                        EventKind::Modify(ModifyKind::Name(rename_mode)) => match rename_mode {
                            RenameMode::From => files_to_rename = event.paths,
                            RenameMode::To => {
                                let mut new_filenames = event.paths;
                                files_to_rename.iter().for_each(
                                    |old_filename| match new_filenames.pop() {
                                        Some(new_filename) => {
                                            if let Err(e) = self.rename(old_filename, &new_filename) {
                                                log::error!("{e}");
                                            }
                                        }
                                        None => log::error!(
                                            "Cannot rename {:?}. Nothing left in the event",
                                            old_filename
                                        ),
                                    },
                                )
                            }
                            _ => log::warn!("rename mode could not be handled: {rename_mode:?}"),
                        },
                        EventKind::Create(_) => {
                            event.paths.iter().for_each(|p| {
                                if let Err(e) = self.copy(p) {
                                    log::error!("{e}");
                                }
                            });
                        }
                        EventKind::Modify(ModifyKind::Any) => {
                            // During directory removal there will be the second MODYFY(ANY) event
                            // causing parent directory to update itself for some reason
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
    use log::{error, LevelFilter};

    fn init() {
        let _ = env_logger::builder()
            .filter_level(LevelFilter::Debug)
            .is_test(true)
            .try_init();
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

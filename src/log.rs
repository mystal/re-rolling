use bevy::prelude::*;
use bevy::log::{Level, LogSettings};

/// Should be before bevy's DefaultPlugins, which will intitialize logging.
pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&self, app: &mut App) {
        // Configure logging.
        if cfg!(feature = "verbose_logs") {
            let mut log_settings = LogSettings::default();
            log_settings.filter.push_str(",info,gmtk_2022=trace");
            log_settings.level = Level::TRACE;
            app.insert_resource(log_settings);
        } else if cfg!(debug_assertions) {
            let mut log_settings = LogSettings::default();
            log_settings.filter.push_str(",info,gmtk_2022=debug");
            log_settings.level = Level::DEBUG;
            app.insert_resource(log_settings);
        }
    }
}

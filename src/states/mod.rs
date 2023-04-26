use bevy::prelude::*;

use self::{device_select::DeviceSelectPlugin, game::GamePlugin, start_menu::StartMenuPlugin};

mod device_select;
pub mod game;
mod start_menu;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    StartMenu,
    DeviceSelect,
    Game,
}

pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<AppState>()
            .add_plugin(GamePlugin)
            .add_plugin(DeviceSelectPlugin)
            .add_plugin(StartMenuPlugin);
    }
}

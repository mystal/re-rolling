use bevy::prelude::*;

pub fn c_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.);
}

pub fn c_timer(b: &mut NodeBundle) {
    let s = &mut b.style;
    s.width = Val::Percent(100.);
    s.position_type = PositionType::Absolute;
    s.top = Val::Px(10.0);
    s.justify_content = JustifyContent::Center;
}

pub fn c_timer_text(_a: &AssetServer, _b: &mut TextBundle) {
}

pub fn c_timer_font(_assets: &AssetServer, s: &mut TextStyle) {
    // s.font = assets.load("fonts/Kenney Pixel.ttf");
    s.font_size = 20.0;
    s.color = Color::WHITE;
}

pub fn c_health_ui(b: &mut NodeBundle) {
    let s = &mut b.style;
    s.position_type = PositionType::Absolute;
    s.top = Val::Px(10.0);
    s.left = Val::Px(10.0);
}

pub fn c_whole_heart(assets: &AssetServer, b: &mut ImageBundle) {
    b.image = assets.load("whole_heart.png").into();
}

//! The sidebar: a left panel with the brand, the nav items and, in a bottom panel of its own, the
//! user. A nav item is a frame that takes its fill from its response, the hover pattern of the
//! `Frame` docs, and a click on it makes it active.

use eframe::egui::{Align, Frame, Layout, Margin, Panel, Sense, Stroke, Ui, Vec2, vec2};

use super::spec;
use super::style::{Weight, area, avatar, block, color, pill, text};

/// The height of the user panel at the bottom, its rule included.
const USER_HEIGHT: f32 = spec::WINDOW.1 - spec::USER_RULE_Y;
/// The space between the left edge of the sidebar and the brand or the user.
const INSET: f32 = 20.0;
/// The top of the user avatar.
const USER_AVATAR_Y: f32 = 740.0;

/// Shows the sidebar; a click on a nav item makes it the `active` one.
pub fn show(ui: &mut Ui, active: &mut usize) {
    Panel::left("sidebar")
        .exact_size(spec::SIDEBAR_WIDTH)
        .resizable(false)
        .show_separator_line(false)
        .frame(Frame::new().fill(color(spec::SIDEBAR_BG)))
        .show(ui, |ui| {
            user(ui);
            brand(ui);
            ui.add_space(spec::NAV_TOP - (brand_top() + spec::LOGO_SIZE));
            Frame::new()
                .inner_margin(Margin::symmetric(spec::NAV_X as i8, 0))
                .show(ui, |ui| {
                    for (i, label) in spec::NAV_ITEMS.iter().enumerate() {
                        if i > 0 {
                            ui.add_space(spec::NAV_STRIDE - spec::NAV_HEIGHT);
                        }
                        nav_item(ui, i, label, active);
                    }
                });
        });
}

/// The top of the logo: centered on the top bar's middle.
fn brand_top() -> f32 {
    spec::TOPBAR_HEIGHT / 2.0 - spec::LOGO_SIZE / 2.0
}

/// The logo square and the app name beside it.
fn brand(ui: &mut Ui) {
    ui.add_space(brand_top());
    let size = vec2(ui.available_width(), spec::LOGO_SIZE);
    area(ui, size, Layout::left_to_right(Align::Center), |ui| {
        ui.add_space(INSET);
        block(
            ui,
            Vec2::splat(spec::LOGO_SIZE),
            spec::LOGO,
            spec::LOGO_RADIUS,
        );
        ui.add_space(12.0);
        ui.label(text(
            spec::APP_NAME,
            spec::TITLE_FONT,
            Weight::Semibold,
            spec::PRIMARY_TEXT,
        ));
    });
}

/// Nav item `i`: its icon, its label and, on the messages item, the unread badge. The fill shows
/// the active item and the hovered one.
fn nav_item(ui: &mut Ui, i: usize, label: &str, active: &mut usize) {
    let is_active = *active == i;
    let mut frame = Frame::new()
        .corner_radius(spec::NAV_RADIUS)
        .inner_margin(Margin {
            left: 16,
            right: 12,
            top: 0,
            bottom: 0,
        })
        .begin(ui);
    {
        let ui = &mut frame.content_ui;
        let size = vec2(ui.available_width(), spec::NAV_HEIGHT);
        area(ui, size, Layout::left_to_right(Align::Center), |ui| {
            let icon = if is_active {
                spec::NAV_ICON_ACTIVE
            } else {
                spec::NAV_ICON_COLOR
            };
            block(ui, Vec2::splat(spec::NAV_ICON), icon, 4);
            ui.add_space(12.0);
            let label_color = if is_active {
                spec::PRIMARY_TEXT
            } else {
                spec::SIDEBAR_TEXT
            };
            ui.label(text(label, spec::FONT_SIZE, Weight::Regular, label_color));
            if i == spec::BADGE_ITEM {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let count = text(
                        spec::BADGE_COUNT.to_string(),
                        12.0,
                        Weight::Semibold,
                        spec::PRIMARY_TEXT,
                    );
                    pill(ui, count, spec::BADGE_BG, 20.0, 7.0);
                });
            }
        });
    }
    let response = frame.allocate_space(ui).interact(Sense::click());
    if response.clicked() {
        *active = i;
    }
    if *active == i {
        frame.frame.fill = color(spec::NAV_ACTIVE);
    } else if response.hovered() {
        frame.frame.fill = color(spec::NAV_HOVER);
    }
    frame.paint(ui);
}

/// The user at the bottom: a bottom panel whose separator line is the rule above it, with the
/// avatar, the name and the role.
fn user(ui: &mut Ui) {
    ui.visuals_mut().widgets.noninteractive.bg_stroke = Stroke::new(1.0, color(spec::SIDEBAR_RULE));
    let inner = Margin {
        left: INSET as i8,
        right: 0,
        // Below the 1 px rule.
        top: (USER_AVATAR_Y - spec::USER_RULE_Y - 1.0) as i8,
        bottom: 0,
    };
    Panel::bottom("user")
        .exact_size(USER_HEIGHT)
        .frame(Frame::new().inner_margin(inner))
        .show(ui, |ui| {
            let size = vec2(ui.available_width(), spec::USER_AVATAR);
            area(ui, size, Layout::left_to_right(Align::Center), |ui| {
                let initials = text(
                    spec::USER_INITIALS,
                    12.0,
                    Weight::Semibold,
                    spec::PRIMARY_TEXT,
                );
                avatar(ui, spec::USER_AVATAR, spec::USER_COLOR, initials);
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(text(
                        spec::USER_NAME,
                        spec::FONT_SIZE,
                        Weight::Regular,
                        spec::PRIMARY_TEXT,
                    ));
                    ui.label(text(
                        spec::USER_ROLE,
                        12.0,
                        Weight::Regular,
                        spec::SIDEBAR_MUTED,
                    ));
                });
            });
        });
}

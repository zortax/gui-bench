//! The look of the dashboard through iced's style functions: colors and fonts of the spec, and
//! the container, button and text input styles that every part shares.

use iced::widget::{button, container, text_input};
use iced::{Background, Border, Font, Shadow, Vector, border, font};

use super::spec;
use crate::page::color;

/// The 600 and 700 weights of the default sans-serif font.
pub const SEMIBOLD: Font = Font {
    weight: font::Weight::Semibold,
    ..Font::DEFAULT
};
pub const BOLD: Font = Font {
    weight: font::Weight::Bold,
    ..Font::DEFAULT
};

/// Returns a shadow of the spec.
pub fn shadow(shadow: spec::Shadow) -> Shadow {
    Shadow {
        color: color(shadow.color).scale_alpha(shadow.alpha),
        offset: Vector::new(shadow.x, shadow.y),
        blur_radius: shadow.blur,
    }
}

/// A filled box with a corner radius.
pub fn filled(background: u32, radius: impl Into<border::Radius>) -> container::Style {
    container::Style {
        background: Some(Background::Color(color(background))),
        border: border::rounded(radius),
        ..container::Style::default()
    }
}

/// A white card with rounded corners and the card shadow.
pub fn card(_: &iced::Theme) -> container::Style {
    container::Style {
        shadow: shadow(spec::SHADOW_CARD),
        ..filled(spec::CARD_BG, spec::CARD_RADIUS)
    }
}

/// The primary button: the primary color, white text and the small shadow.
pub fn primary(_: &iced::Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => color(spec::PRIMARY).scale_alpha(0.9),
        button::Status::Active | button::Status::Disabled => color(spec::PRIMARY),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: color(spec::PRIMARY_TEXT),
        border: border::rounded(8),
        shadow: shadow(spec::SHADOW_SMALL),
        snap: true,
    }
}

/// The secondary button: white with a 1 px border.
pub fn secondary(_: &iced::Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => color(spec::INPUT_BG),
        button::Status::Active | button::Status::Disabled => color(spec::SECONDARY_BG),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: color(spec::TEXT),
        border: Border {
            color: color(spec::SECONDARY_BORDER),
            width: 1.0,
            radius: 8.0.into(),
        },
        shadow: Shadow::default(),
        snap: true,
    }
}

/// The search input: a light fill, a 1 px border that turns primary on focus.
pub fn input(_: &iced::Theme, status: text_input::Status) -> text_input::Style {
    let border = match status {
        text_input::Status::Focused { .. } => spec::PRIMARY,
        _ => spec::INPUT_BORDER,
    };
    text_input::Style {
        background: Background::Color(color(spec::INPUT_BG)),
        border: Border {
            color: color(border),
            width: 1.0,
            radius: 8.0.into(),
        },
        icon: color(spec::MUTED),
        placeholder: color(spec::SIDEBAR_MUTED),
        value: color(spec::TEXT),
        selection: color(spec::SPARK),
    }
}

//! The text font: the regular face of the system `sans-serif` family comes first in the
//! proportional family, ahead of the fonts that egui bundles.

use std::sync::Arc;

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use fontique::{Collection, CollectionOptions, FontStyle, FontWeight, FontWidth, GenericFamily};

const NAME: &str = "system-sans-serif";

/// Returns the egui font definitions with the system sans-serif face in front, or the egui
/// defaults when the system has no such face.
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let Some(data) = system_sans_serif() else {
        eprintln!("egui-bench: no system sans-serif font, using the egui default font");
        return fonts;
    };
    fonts.font_data.insert(NAME.to_owned(), Arc::new(data));
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, NAME.to_owned());
    fonts
}

/// Loads the regular face of the first family that fontconfig maps to `sans-serif`.
fn system_sans_serif() -> Option<FontData> {
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: true,
    });
    let ids: Vec<_> = collection
        .generic_families(GenericFamily::SansSerif)
        .collect();
    ids.into_iter().find_map(|id| {
        let family = collection.family(id)?;
        let font = family.match_font(
            FontWidth::NORMAL,
            FontStyle::Normal,
            FontWeight::NORMAL,
            false,
        )?;
        let blob = font.load(None)?;
        let mut data = FontData::from_owned(blob.as_ref().to_vec());
        data.index = font.index();
        Some(data)
    })
}

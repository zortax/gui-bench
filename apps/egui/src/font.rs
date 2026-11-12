//! The text font: the regular face of the system `sans-serif` family comes first in the
//! proportional family, ahead of the fonts that egui bundles. egui has no font weights, so a
//! bolder face is a named family of its own (`with_weights`), as egui's font docs teach for any
//! extra font.

use std::sync::Arc;

use eframe::egui::{FontData, FontDefinitions, FontFamily};
use fontique::{
    Collection, CollectionOptions, FamilyId, FontInfo, FontStyle, FontWeight, FontWidth,
    GenericFamily,
};

const NAME: &str = "system-sans-serif";

/// Returns the egui font definitions with the system sans-serif face in front, or the egui
/// defaults when the system has no such face.
pub fn definitions() -> FontDefinitions {
    with_weights(&[])
}

/// Returns the definitions of `definitions`, plus one named family per `(name, weight)`: the
/// system sans-serif face that fontconfig matches to the weight, then the proportional fonts.
pub fn with_weights(weights: &[(&str, FontWeight)]) -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut collection = Collection::new(CollectionOptions {
        shared: false,
        system_fonts: true,
    });
    let ids: Vec<_> = collection
        .generic_families(GenericFamily::SansSerif)
        .collect();
    // The faces loaded so far, by source and index, so two weights that match one file share it.
    let mut loaded: Vec<(FontInfo, String)> = Vec::new();
    let mut load = |weight: FontWeight, key: String, fonts: &mut FontDefinitions| {
        let font = find(&mut collection, &ids, weight)?;
        if let Some((_, key)) = loaded
            .iter()
            .find(|(f, _)| f.source().id() == font.source().id() && f.index() == font.index())
        {
            return Some(key.clone());
        }
        let blob = font.load(None)?;
        let mut data = FontData::from_owned(blob.as_ref().to_vec());
        data.index = font.index();
        fonts.font_data.insert(key.clone(), Arc::new(data));
        loaded.push((font, key.clone()));
        Some(key)
    };
    match load(FontWeight::NORMAL, NAME.to_owned(), &mut fonts) {
        Some(key) => fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, key),
        None => eprintln!("egui-bench: no system sans-serif font, using the egui default font"),
    }
    let proportional = fonts.families[&FontFamily::Proportional].clone();
    for &(name, weight) in weights {
        let mut family = proportional.clone();
        if let Some(key) = load(weight, format!("{NAME}-{name}"), &mut fonts) {
            family.insert(0, key);
        }
        fonts.families.insert(FontFamily::Name(name.into()), family);
    }
    fonts
}

/// Finds the face closest to `weight` in the first family that fontconfig maps to `sans-serif`.
fn find(collection: &mut Collection, ids: &[FamilyId], weight: FontWeight) -> Option<FontInfo> {
    ids.iter().find_map(|&id| {
        collection
            .family(id)?
            .match_font(FontWidth::NORMAL, FontStyle::Normal, weight, false)
            .cloned()
    })
}

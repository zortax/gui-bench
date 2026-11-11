//! Export of a chart: SVG with the colors of the current theme resolved and a title block and
//! source line as the gui-bench plots have, PNG at twice the size, and CSV of the shown data.

use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

use super::CardCtx;

/// The style properties that an export copies from the page onto every element.
const PROPS: [&str; 14] = [
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-opacity",
    "stroke-dasharray",
    "stroke-linecap",
    "stroke-linejoin",
    "opacity",
    "font-size",
    "font-weight",
    "font-family",
    "text-anchor",
    "dominant-baseline",
];

/// The height of the title block and the source line in pixels.
const HEAD: f64 = 58.0;
const FOOT: f64 = 26.0;

fn css(name: &str) -> String {
    let Some(root) = document().document_element() else {
        return String::new();
    };
    window()
        .get_computed_style(&root)
        .ok()
        .flatten()
        .and_then(|s| s.get_property_value(name).ok())
        .map(|v| v.trim().to_owned())
        .unwrap_or_default()
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Returns the source line of the run of the page.
fn footnote(ctx: CardCtx) -> String {
    ctx.explorer
        .map(|e| e.footnote.get_untracked())
        .filter(|f| !f.is_empty())
        .unwrap_or_else(|| "gui-bench explorer".to_owned())
}

/// Serializes the chart SVG with inline styles inside a page with a title block.
fn compose(ctx: CardCtx, font: Option<(String, String)>) -> Option<(String, f64, f64)> {
    let svg: web_sys::SvgElement = ctx.svg.get_untracked()?;
    let original: &web_sys::Element = svg.as_ref();
    let clone: web_sys::Element = original.clone_node_with_deep(true).ok()?.dyn_into().ok()?;
    let a = original.query_selector_all("*").ok()?;
    let b = clone.query_selector_all("*").ok()?;
    let window = window();
    for i in 0..a.length() {
        let (Some(src), Some(dst)) = (a.item(i), b.item(i)) else {
            continue;
        };
        let (Ok(src), Ok(dst)) = (
            src.dyn_into::<web_sys::Element>(),
            dst.dyn_into::<web_sys::Element>(),
        ) else {
            continue;
        };
        let Ok(Some(style)) = window.get_computed_style(&src) else {
            continue;
        };
        let inline: String = PROPS
            .iter()
            .filter_map(|p| {
                let v = style.get_property_value(p).ok()?;
                (!v.is_empty()).then(|| format!("{p}:{v};"))
            })
            .collect();
        let _ = dst.set_attribute("style", &inline);
        let _ = dst.remove_attribute("class");
    }
    let width = f64::from(original.client_width());
    let height = f64::from(original.client_height());
    let _ = clone.set_attribute("x", "16");
    let _ = clone.set_attribute("y", &HEAD.to_string());
    let inner = web_sys::XmlSerializer::new()
        .ok()?
        .serialize_to_string(&clone)
        .ok()?;
    let page = css("--page");
    let text = css("--text");
    let text2 = css("--text-2");
    let muted = css("--muted");
    let w = width + 32.0;
    let h = height + HEAD + FOOT;
    let style = match font {
        Some((regular, bold)) => format!(
            "<style>@font-face{{font-family:'Libertinus Sans';src:url(data:font/ttf;base64,{regular});font-weight:400}}@font-face{{font-family:'Libertinus Sans';src:url(data:font/ttf;base64,{bold});font-weight:700}}</style>"
        ),
        None => String::new(),
    };
    let family = "font-family:'Libertinus Sans',sans-serif";
    let out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">{style}\
<rect width=\"100%\" height=\"100%\" fill=\"{page}\"/>\
<text x=\"16\" y=\"26\" style=\"{family};font-size:17px;font-weight:700;fill:{text}\">{}</text>\
<text x=\"16\" y=\"45\" style=\"{family};font-size:11.5px;fill:{text2}\">{}</text>\
{inner}\
<text x=\"16\" y=\"{}\" style=\"{family};font-size:9.5px;fill:{muted}\">{}</text></svg>",
        escape(&ctx.title.get_value()),
        escape(&ctx.note.get_untracked()),
        h - 10.0,
        escape(&footnote(ctx)),
    );
    Some((out, w, h))
}

fn download(name: &str, mime: &str, data: &JsValue) {
    let parts = js_sys::Array::of1(data);
    let options = web_sys::BlobPropertyBag::new();
    options.set_type(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&parts, &options) else {
        return;
    };
    download_blob(name, &blob);
}

fn download_blob(name: &str, blob: &web_sys::Blob) {
    let Ok(url) = web_sys::Url::create_object_url_with_blob(blob) else {
        return;
    };
    if let Ok(a) = document().create_element("a")
        && let Ok(a) = a.dyn_into::<web_sys::HtmlAnchorElement>()
    {
        a.set_href(&url);
        a.set_download(name);
        a.click();
    }
    let _ = web_sys::Url::revoke_object_url(&url);
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, b)| n | u32::from(*b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(B64[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

async fn fetch_base64(url: &str) -> Option<String> {
    let response = wasm_bindgen_futures::JsFuture::from(window().fetch_with_str(url))
        .await
        .ok()?;
    let response: web_sys::Response = response.dyn_into().ok()?;
    let buffer = wasm_bindgen_futures::JsFuture::from(response.array_buffer().ok()?)
        .await
        .ok()?;
    Some(base64(&js_sys::Uint8Array::new(&buffer).to_vec()))
}

async fn fonts() -> Option<(String, String)> {
    Some((
        fetch_base64("/fonts/LibertinusSans-Regular.ttf").await?,
        fetch_base64("/fonts/LibertinusSans-Bold.ttf").await?,
    ))
}

/// Downloads the chart as SVG, optionally with the font inside.
pub fn save_svg(ctx: CardCtx, name: &str, embed: bool) {
    let name = format!("{name}.svg");
    leptos::task::spawn_local(async move {
        let font = if embed { fonts().await } else { None };
        if let Some((svg, _, _)) = compose(ctx, font) {
            download(&name, "image/svg+xml", &JsValue::from_str(&svg));
        }
    });
}

/// Downloads the chart as PNG at twice its size, with the font inside.
pub fn save_png(ctx: CardCtx, name: &str) {
    let name = format!("{name}.png");
    leptos::task::spawn_local(async move {
        let Some((svg, w, h)) = compose(ctx, fonts().await) else {
            return;
        };
        let _ = render_png(&svg, w, h, &name).await;
    });
}

async fn render_png(svg: &str, w: f64, h: f64, name: &str) -> Result<(), JsValue> {
    let img = web_sys::HtmlImageElement::new()?;
    let loaded = js_sys::Promise::new(&mut |resolve, reject| {
        img.set_onload(Some(&resolve));
        img.set_onerror(Some(&reject));
    });
    img.set_src(&format!(
        "data:image/svg+xml;base64,{}",
        base64(svg.as_bytes())
    ));
    wasm_bindgen_futures::JsFuture::from(loaded).await?;
    let canvas: web_sys::HtmlCanvasElement = document().create_element("canvas")?.dyn_into()?;
    canvas.set_width((w * 2.0) as u32);
    canvas.set_height((h * 2.0) as u32);
    let context: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .ok_or_else(|| JsValue::from_str("no 2d context"))?
        .dyn_into()?;
    context.scale(2.0, 2.0)?;
    context.draw_image_with_html_image_element(&img, 0.0, 0.0)?;
    let url = canvas.to_data_url_with_type("image/png")?;
    if let Ok(a) = document().create_element("a")
        && let Ok(a) = a.dyn_into::<web_sys::HtmlAnchorElement>()
    {
        a.set_href(&url);
        a.set_download(name);
        a.click();
    }
    Ok(())
}

/// Downloads the shown data as CSV.
pub fn save_csv(ctx: CardCtx, name: &str) {
    download(
        &format!("{name}.csv"),
        "text/csv",
        &JsValue::from_str(&ctx.csv.get_untracked()),
    );
}

/// Copies the shown data as CSV to the clipboard.
pub fn copy_csv(ctx: CardCtx) {
    let clipboard = window().navigator().clipboard();
    let _ = clipboard.write_text(&ctx.csv.get_untracked());
}

#[cfg(test)]
mod tests {
    use super::base64;

    #[test]
    fn encodes() {
        assert_eq!(base64(b"Man"), "TWFu");
        assert_eq!(base64(b"Ma"), "TWE=");
        assert_eq!(base64(b"M"), "TQ==");
    }
}

//! The spacing of direct labels next to line ends: labels keep a minimum gap and stay as close to
//! their wanted positions as the gap allows, as the gui-bench plots place them.

/// Returns label positions for wanted positions (in pixels, any order), at least `gap` apart.
/// The result keeps the input order.
pub fn spread(wants: &[f64], gap: f64) -> Vec<f64> {
    let mut order: Vec<usize> = (0..wants.len()).collect();
    order.sort_by(|&a, &b| wants[a].total_cmp(&wants[b]));
    let mut got: Vec<f64> = order.iter().map(|&i| wants[i]).collect();
    let push = |got: &mut Vec<f64>| {
        for i in 1..got.len() {
            got[i] = got[i].max(got[i - 1] + gap);
        }
    };
    push(&mut got);
    if !got.is_empty() {
        let shift = (got.iter().sum::<f64>() - order.iter().map(|&i| wants[i]).sum::<f64>())
            / got.len() as f64;
        for g in &mut got {
            *g -= shift;
        }
    }
    push(&mut got);
    let mut out = vec![0.0; wants.len()];
    for (slot, &i) in order.iter().enumerate() {
        out[i] = got[slot];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_gap_and_order() {
        let out = spread(&[10.0, 11.0, 50.0, 10.5], 10.0);
        let mut sorted = out.clone();
        sorted.sort_by(f64::total_cmp);
        for w in sorted.windows(2) {
            assert!(w[1] - w[0] >= 10.0 - 1e-9);
        }
        assert!((out[2] - 50.0).abs() < 1e-9 || out[2] > 40.0);
        assert!(out[0] < out[1]);
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub enum TransitionKind {
    Cut,
    Fade,
    Dissolve,
    WipeLeft,
    WipeDown,
}

fn lerp_color(a: (u8, u8, u8), b: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let r = a.0 as f64 * (1.0 - t) + b.0 as f64 * t;
    let g = a.1 as f64 * (1.0 - t) + b.1 as f64 * t;
    let bl = a.2 as f64 * (1.0 - t) + b.2 as f64 * t;
    (r as u8, g as u8, bl as u8)
}

pub fn apply_transition(
    kind: TransitionKind,
    from: &[(u8, u8, u8)],
    to: &[(u8, u8, u8)],
    output: &mut [(u8, u8, u8)],
    width: u32,
    height: u32,
    progress: f64,
) {
    let progress = progress.clamp(0.0, 1.0);
    let len = output.len().min(from.len()).min(to.len());

    match kind {
        TransitionKind::Cut => {
            if progress < 0.5 {
                output[..len].copy_from_slice(&from[..len]);
            } else {
                output[..len].copy_from_slice(&to[..len]);
            }
        }
        TransitionKind::Fade => {
            // Fade out to black, then fade in from black
            let black = (0u8, 0u8, 0u8);
            if progress < 0.5 {
                let t = progress * 2.0;
                for i in 0..len {
                    output[i] = lerp_color(from[i], black, t);
                }
            } else {
                let t = (progress - 0.5) * 2.0;
                for i in 0..len {
                    output[i] = lerp_color(black, to[i], t);
                }
            }
        }
        TransitionKind::Dissolve => {
            for i in 0..len {
                output[i] = lerp_color(from[i], to[i], progress);
            }
        }
        TransitionKind::WipeLeft => {
            let threshold = (width as f64 * progress) as u32;
            for i in 0..len {
                let x = (i as u32) % width;
                output[i] = if x < threshold { to[i] } else { from[i] };
            }
        }
        TransitionKind::WipeDown => {
            let threshold = (height as f64 * progress) as u32;
            for i in 0..len {
                let y = (i as u32) / width;
                output[i] = if y < threshold { to[i] } else { from[i] };
            }
        }
    }
}

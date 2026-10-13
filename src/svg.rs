//! Standalone SVG and an offline HTML stage viewer, with no external assets.
use crate::trace::Trace;
use std::{fmt::Write as _, fs, io, path::Path};

fn position(mask: usize, n: usize) -> (i32, i32) {
    let mut x = 130;
    let mut y = 350;
    for (bit, (dx, dy)) in [(300, 0), (0, -230), (170, -90)].iter().enumerate().take(n) {
        if mask & (1 << bit) != 0 {
            x += dx;
            y += dy;
        }
    }
    (x, y)
}

/// Render an n<=3 cube; highlighted arrows identify the current dimension.
pub fn stage_svg(trace: &Trace, stage: usize) -> String {
    let n = trace.states.len() - 1;
    assert!(n <= 3, "cube diagrams support at most 3 dimensions");
    assert!(stage <= n, "invalid stage");
    let values = &trace.states[stage];
    let heading = if stage == 0 {
        "Initial values".to_owned()
    } else {
        format!("After bit {}", stage - 1)
    };
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 760 540" role="img" aria-label="{heading}">
<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M 0 0 L 10 5 L 0 10 z" fill="#007f78"/></marker></defs>
<rect width="760" height="540" fill="#f8fafc"/><g font-family="system-ui,sans-serif"><text x="28" y="28" font-size="20" font-weight="bold">{heading}</text>
"##
    );
    // Shift the drawing down to leave a clear title above the cube.
    svg.push_str("<g transform=\"translate(0,50)\">");
    for mask in 0..values.len() {
        for bit in 0..n {
            if mask & (1 << bit) != 0 {
                continue;
            }
            let (x, y) = position(mask, n);
            let (xx, yy) = position(mask | (1 << bit), n);
            let active = stage == bit + 1;
            let color = if active { "#007f78" } else { "#cbd5e1" };
            let marker = if active {
                " marker-end=\"url(#arrow)\""
            } else {
                ""
            };
            let dx = (xx - x) as f64;
            let dy = (yy - y) as f64;
            let scale = 36.0 / dx.hypot(dy);
            writeln!(svg, r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{color}" stroke-width="{}"{marker}/ >"#,
                x as f64 + dx * scale, y as f64 + dy * scale,
                xx as f64 - dx * scale, yy as f64 - dy * scale,
                if active { 4 } else { 2 }).unwrap();
        }
    }
    for (mask, value) in values.iter().enumerate() {
        let (x, y) = position(mask, n);
        let bits = format!("{mask:0width$b}", width = n.max(1));
        writeln!(svg, r##"<circle cx="{x}" cy="{y}" r="33" fill="white" stroke="#334155"/><text x="{x}" y="{}" text-anchor="middle" font-size="12" fill="#475569">{bits}</text><text x="{x}" y="{}" text-anchor="middle" font-size="18" font-weight="bold">{value}</text>"##, y - 8, y + 15).unwrap();
    }
    svg.push_str("</g>");
    if stage > 0 {
        let bit = 1 << (stage - 1);
        let previous = &trace.states[stage - 1];
        let sign = if trace.inverse { "−" } else { "+" };
        let mut row = 0;
        for mask in 0..values.len() {
            if mask & bit != 0 {
                let column = row % 2;
                let y = 468 + (row / 2) * 26;
                writeln!(svg, r#"<text x="{}" y="{y}" font-size="14">{:0width$b}: {} {sign} ({}) = {}</text>"#,
                    28 + column * 370, mask, previous[mask], previous[mask ^ bit], values[mask], width=n.max(1)).unwrap();
                row += 1;
            }
        }
    }
    svg.push_str("</g></svg>");
    svg.replace("/ >", "/>")
}

/// Write individual stage SVGs and a keyboard-accessible stage selector.
pub fn write_report(trace: &Trace, directory: &Path) -> io::Result<()> {
    if trace.states.len() > 4 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "SVG supports n<=3",
        ));
    }
    fs::create_dir_all(directory)?;
    let mut html = String::from(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>Boolean cube — Fast Zeta Transform</title><style>body{font:16px system-ui;max-width:900px;margin:40px auto;padding:0 20px;background:#f8fafc;color:#0f172a}img{width:100%}select{font:inherit;padding:8px}</style><h1>Values flowing through the Boolean cube</h1><p>Each mask is a subset. The rightmost bit is bit 0. An active arrow carries the source value into a destination containing that bit.</p><label for=\"stage\">Transformation stage </label><select id=\"stage\">",
    );
    for stage in 0..trace.states.len() {
        fs::write(
            directory.join(format!("stage-{stage}.svg")),
            stage_svg(trace, stage),
        )?;
        write!(
            html,
            "<option value=\"{stage}\">{}</option>",
            if stage == 0 {
                "Initial".to_owned()
            } else {
                format!("After bit {}", stage - 1)
            }
        )
        .unwrap();
    }
    html.push_str("</select><img id=\"cube\" src=\"stage-0.svg\" alt=\"Boolean cube, initial values\"><p>Below each stage, the equations show every active destination update. All files work offline.</p><script>document.getElementById('stage').addEventListener('change',function(){const image=document.getElementById('cube');image.src='stage-'+this.value+'.svg';image.alt='Boolean cube, '+this.options[this.selectedIndex].text;});</script></html>");
    fs::write(directory.join("index.html"), html)
}

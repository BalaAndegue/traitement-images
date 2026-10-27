//! Design tokens — graphite + accent teal (look outil premium).

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Vec2};

pub const BG: Color32 = Color32::from_rgb(12, 14, 18);
pub const SURFACE: Color32 = Color32::from_rgb(18, 21, 27);
pub const SURFACE_2: Color32 = Color32::from_rgb(24, 28, 36);
pub const SURFACE_3: Color32 = Color32::from_rgb(32, 38, 48);
pub const BORDER: Color32 = Color32::from_rgb(42, 48, 60);
pub const TEXT: Color32 = Color32::from_rgb(236, 240, 245);
pub const TEXT_DIM: Color32 = Color32::from_rgb(140, 150, 165);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(90, 100, 115);
pub const ACCENT: Color32 = Color32::from_rgb(45, 212, 191);
pub const ACCENT_DIM: Color32 = Color32::from_rgb(20, 120, 110);
pub const ACCENT_SOFT: Color32 = Color32::from_rgb(22, 48, 48);
pub const DANGER: Color32 = Color32::from_rgb(248, 113, 113);
pub const SUCCESS: Color32 = Color32::from_rgb(74, 222, 128);
pub const HIST_TOP: Color32 = Color32::from_rgb(94, 234, 212);
pub const HIST_BOT: Color32 = Color32::from_rgb(45, 120, 180);

pub const RADIUS: f32 = 12.0;
pub const SIDEBAR_W: f32 = 72.0;
pub const INSPECTOR_W: f32 = 300.0;

pub fn appliquer(ctx: &egui::Context) {
    charger_polices(ctx);

    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = BG;
    visuals.panel_fill = SURFACE;
    visuals.extreme_bg_color = BG;
    visuals.faint_bg_color = SURFACE_2;
    visuals.code_bg_color = SURFACE_2;
    visuals.override_text_color = Some(TEXT);
    visuals.hyperlink_color = ACCENT;
    visuals.selection.bg_fill = ACCENT_SOFT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.window_corner_radius = CornerRadius::same(RADIUS as u8);
    visuals.menu_corner_radius = CornerRadius::same(10);
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.bg_fill = SURFACE_2;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_DIM);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(8);
    visuals.widgets.inactive.bg_fill = SURFACE_3;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 48, 60);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT_DIM);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
    visuals.widgets.active.bg_fill = ACCENT_DIM;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, TEXT);
    visuals.widgets.active.corner_radius = CornerRadius::same(8);
    visuals.widgets.open.bg_fill = SURFACE_3;
    visuals.widgets.open.bg_stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.open.corner_radius = CornerRadius::same(8);
    visuals.slider_trailing_fill = true;
    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(0);
    style.spacing.indent = 16.0;
    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(22.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(13.5, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );
    style.visuals = ctx.style().visuals.clone();
    ctx.set_style(style);
}

fn charger_polices(ctx: &egui::Context) {
    // Segoe UI si disponible (Windows) — sinon police egui par défaut.
    let candidates = [
        r"C:\Windows\Fonts\segoeui.ttf",
        r"C:\Windows\Fonts\SegoeUI.ttf",
        "/System/Library/Fonts/SFNS.ttf",
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
    ];
    let Some(bytes) = candidates
        .iter()
        .find_map(|p| std::fs::read(p).ok())
    else {
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "ui".into(),
        egui::FontData::from_owned(bytes).into(),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "ui".into());
    ctx.set_fonts(fonts);
}

pub fn carte() -> egui::Frame {
    egui::Frame::new()
        .fill(SURFACE_2)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS)
        .inner_margin(egui::Margin::same(16))
}

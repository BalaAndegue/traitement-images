//! Interface graphique premium (Option A) — shell type outil photo moderne.
//!
//! Opérations branchées sur `imagerie::ponctuel` et `imagerie::arithmetique`.

mod theme;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use eframe::egui::{
    self, Color32, ColorImage, CornerRadius, Mesh, Pos2, Rect, Sense, Shape, Stroke, TextureHandle,
    Vec2,
};
use imagerie::arithmetique;
use imagerie::es;
use imagerie::image::ImageGris;
use imagerie::ponctuel;
use theme::{
    ACCENT, ACCENT_SOFT, BG, BORDER, DANGER, HIST_BOT, HIST_TOP, INSPECTOR_W, SIDEBAR_W, SUCCESS,
    SURFACE, SURFACE_2, SURFACE_3, TEXT, TEXT_DIM, TEXT_MUTED,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("Imagerie"),
        ..Default::default()
    };
    eframe::run_native(
        "Imagerie",
        options,
        Box::new(|cc| Ok(Box::new(App::nouvelle(cc)))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Chapitre {
    Ponctuel,
    Arithmetique,
}

impl Chapitre {
    fn toutes() -> &'static [Chapitre] {
        &[Self::Ponctuel, Self::Arithmetique]
    }

    fn titre(self) -> &'static str {
        match self {
            Self::Ponctuel => "Contraste",
            Self::Arithmetique => "Arithmétique",
        }
    }

    fn sous_titre(self) -> &'static str {
        match self {
            Self::Ponctuel => "Chapitre 2 — histogramme & transformations",
            Self::Arithmetique => "Chapitre 2 — opérations sur deux images",
        }
    }

    fn glyphe(self) -> &'static str {
        match self {
            Self::Ponctuel => "▦",
            Self::Arithmetique => "＋",
        }
    }

    fn operations(self) -> &'static [Operation] {
        match self {
            Self::Ponctuel => &[
                Operation::Negatif,
                Operation::Lineaire,
                Operation::Saturation,
                Operation::Gamma,
                Operation::Log,
                Operation::Exp,
                Operation::Egalisation,
                Operation::EgalisationLocale,
            ],
            Self::Arithmetique => &[
                Operation::Addition,
                Operation::Soustraction,
                Operation::DiffAbs,
                Operation::Multiplication,
                Operation::Combinaison,
                Operation::Et,
                Operation::Ou,
            ],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    Negatif,
    Lineaire,
    Saturation,
    Gamma,
    Log,
    Exp,
    Egalisation,
    EgalisationLocale,
    Addition,
    Soustraction,
    DiffAbs,
    Multiplication,
    Combinaison,
    Et,
    Ou,
}

impl Operation {
    fn label(self) -> &'static str {
        match self {
            Self::Negatif => "Négatif",
            Self::Lineaire => "Étirement linéaire",
            Self::Saturation => "Linéaire + saturation",
            Self::Gamma => "Correction gamma",
            Self::Log => "Logarithme",
            Self::Exp => "Exponentielle",
            Self::Egalisation => "Égalisation",
            Self::EgalisationLocale => "Égalisation locale 7×7",
            Self::Addition => "Addition",
            Self::Soustraction => "Soustraction",
            Self::DiffAbs => "Différence absolue",
            Self::Multiplication => "Multiplication",
            Self::Combinaison => "Combinaison linéaire",
            Self::Et => "ET bit à bit",
            Self::Ou => "OU bit à bit",
        }
    }

    fn besoin_image_b(self) -> bool {
        matches!(
            self,
            Self::Addition
                | Self::Soustraction
                | Self::DiffAbs
                | Self::Combinaison
                | Self::Et
                | Self::Ou
        )
    }
}

#[derive(Clone)]
struct Toast {
    texte: String,
    ok: bool,
    cree: Instant,
}

struct Stats {
    min: u8,
    max: u8,
    moyenne: f32,
}

struct App {
    chapitre: Chapitre,
    operation: Operation,
    s_min: f32,
    s_max: f32,
    gamma: f32,
    ratio: f32,
    alpha: f32,
    chemin: Option<PathBuf>,
    chemin_b: Option<PathBuf>,
    entree: Option<ImageGris>,
    entree_b: Option<ImageGris>,
    sortie: Option<ImageGris>,
    tex_entree: Option<TextureHandle>,
    tex_sortie: Option<TextureHandle>,
    hist_entree: ponctuel::Histogramme,
    hist_sortie: ponctuel::Histogramme,
    stats_entree: Option<Stats>,
    stats_sortie: Option<Stats>,
    /// 0 = tout entrée, 1 = tout sortie
    split: f32,
    anim_split: f32,
    toast: Option<Toast>,
    hover_drop: bool,
    pulse: f32,
}

impl App {
    fn nouvelle(cc: &eframe::CreationContext<'_>) -> Self {
        theme::appliquer(&cc.egui_ctx);
        Self {
            chapitre: Chapitre::Ponctuel,
            operation: Operation::Negatif,
            s_min: 40.0,
            s_max: 200.0,
            gamma: 1.0,
            ratio: 1.2,
            alpha: 0.5,
            chemin: None,
            chemin_b: None,
            entree: None,
            entree_b: None,
            sortie: None,
            tex_entree: None,
            tex_sortie: None,
            hist_entree: [0; 256],
            hist_sortie: [0; 256],
            stats_entree: None,
            stats_sortie: None,
            split: 0.5,
            anim_split: 0.5,
            toast: None,
            hover_drop: false,
            pulse: 0.0,
        }
    }

    fn toast(&mut self, texte: impl Into<String>, ok: bool) {
        self.toast = Some(Toast {
            texte: texte.into(),
            ok,
            cree: Instant::now(),
        });
    }

    fn charger_chemin(&mut self, ctx: &egui::Context, chemin: PathBuf) {
        match es::charger_gris(&chemin) {
            Ok(img) => {
                let (l, h) = img.dimensions();
                self.hist_entree = ponctuel::histogramme(&img);
                self.stats_entree = Some(stats_depuis_lib(&img));
                self.tex_entree = Some(texture_depuis_gris(ctx, "entree", &img));
                self.entree = Some(img);
                self.sortie = None;
                self.tex_sortie = None;
                self.hist_sortie = [0; 256];
                self.stats_sortie = None;
                self.split = 0.5;
                self.anim_split = 0.5;
                self.chemin = Some(chemin);
                self.toast(format!("Image A chargée · {l}×{h}"), true);
            }
            Err(e) => self.toast(format!("Impossible de lire le fichier : {e}"), false),
        }
    }

    fn charger_chemin_b(&mut self, chemin: PathBuf) {
        match es::charger_gris(&chemin) {
            Ok(img) => {
                let (l, h) = img.dimensions();
                self.entree_b = Some(img);
                self.chemin_b = Some(chemin);
                self.toast(format!("Image B chargée · {l}×{h}"), true);
            }
            Err(e) => self.toast(format!("Impossible de lire l'image B : {e}"), false),
        }
    }

    fn dialogue_ouvrir(secondaire: bool) -> Option<PathBuf> {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("Images", &["pgm", "ppm", "png", "jpg", "jpeg", "bmp"]);
        if Path::new("images").is_dir() {
            dialog = dialog.set_directory("images");
        }
        let _ = secondaire;
        dialog.pick_file()
    }

    fn ouvrir_dialogue(&mut self, ctx: &egui::Context) {
        if let Some(chemin) = Self::dialogue_ouvrir(false) {
            self.charger_chemin(ctx, chemin);
        }
    }

    fn ouvrir_dialogue_b(&mut self) {
        if let Some(chemin) = Self::dialogue_ouvrir(true) {
            self.charger_chemin_b(chemin);
        }
    }

    fn sync_operations(&mut self) {
        if !self.chapitre.operations().contains(&self.operation) {
            self.operation = self.chapitre.operations()[0];
        }
    }

    fn appliquer(&mut self, ctx: &egui::Context) {
        let Some(entree) = self.entree.clone() else {
            self.toast("Ouvre d’abord une image A", false);
            return;
        };

        if self.operation.besoin_image_b() && self.entree_b.is_none() {
            self.toast("Cette opération nécessite une image B", false);
            return;
        }

        let resultat = match self.operation {
            Operation::Negatif => Ok(ponctuel::negatif(&entree)),
            Operation::Lineaire => Ok(ponctuel::transformation_lineaire(&entree)),
            Operation::Saturation => ponctuel::transformation_lineaire_saturation(
                &entree,
                self.s_min.round().clamp(0.0, 255.0) as u8,
                self.s_max.round().clamp(0.0, 255.0) as u8,
            ),
            Operation::Gamma => ponctuel::correction_gamma(&entree, self.gamma as f64),
            Operation::Log => Ok(ponctuel::transformation_log(&entree)),
            Operation::Exp => Ok(ponctuel::transformation_exp(&entree)),
            Operation::Egalisation => Ok(ponctuel::egalisation_histogramme(&entree)),
            Operation::EgalisationLocale => ponctuel::egalisation_locale(&entree, 7),
            Operation::Multiplication => {
                arithmetique::multiplication(&entree, self.ratio as f64)
            }
            Operation::Addition => {
                arithmetique::addition(&entree, self.entree_b.as_ref().unwrap())
            }
            Operation::Soustraction => {
                arithmetique::soustraction(&entree, self.entree_b.as_ref().unwrap())
            }
            Operation::DiffAbs => {
                arithmetique::difference_absolue(&entree, self.entree_b.as_ref().unwrap())
            }
            Operation::Combinaison => arithmetique::combinaison_lineaire(
                &entree,
                self.entree_b.as_ref().unwrap(),
                self.alpha as f64,
            ),
            Operation::Et => arithmetique::et(&entree, self.entree_b.as_ref().unwrap()),
            Operation::Ou => arithmetique::ou(&entree, self.entree_b.as_ref().unwrap()),
        };

        match resultat {
            Ok(resultat) => {
                self.hist_sortie = ponctuel::histogramme(&resultat);
                self.stats_sortie = Some(stats_depuis_lib(&resultat));
                self.tex_sortie = Some(texture_depuis_gris(ctx, "sortie", &resultat));
                self.sortie = Some(resultat);
                self.split = 0.55;
                self.toast(format!("{} appliqué", self.operation.label()), true);
            }
            Err(e) => self.toast(format!("{e}"), false),
        }
    }

    fn consommer_drop(&mut self, ctx: &egui::Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        self.hover_drop = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if let Some(chemin) = dropped.into_iter().next() {
            self.charger_chemin(ctx, chemin);
        }
    }

    fn peut_appliquer(&self) -> bool {
        self.entree.is_some()
            && (!self.operation.besoin_image_b() || self.entree_b.is_some())
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.consommer_drop(ctx);
        self.sync_operations();

        let dt = ctx.input(|i| i.stable_dt).clamp(0.0, 0.05);
        self.anim_split += (self.split - self.anim_split) * (1.0 - (-12.0 * dt).exp());
        self.pulse = (self.pulse + dt * 2.2) % std::f32::consts::TAU;
        if self.toast.is_some() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if (self.anim_split - self.split).abs() > 0.001 {
            ctx.request_repaint();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG).inner_margin(0.0))
            .show(ctx, |ui| {
                let full = ui.available_rect_before_wrap();

                // Sidebar
                let side = Rect::from_min_size(full.min, Vec2::new(SIDEBAR_W, full.height()));
                self.ui_sidebar(ui, side);

                // Inspector
                let insp = Rect::from_min_max(
                    Pos2::new(full.max.x - INSPECTOR_W, full.min.y),
                    full.max,
                );
                self.ui_inspector(ui, insp);

                // Main
                let main = Rect::from_min_max(
                    Pos2::new(side.max.x, full.min.y),
                    Pos2::new(insp.min.x, full.max.y),
                );
                self.ui_main(ui, main);

                self.ui_toast(ui, full);
            });
    }
}

impl App {
    fn ui_sidebar(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.painter()
                .rect_filled(rect, 0.0, SURFACE);
            ui.painter().line_segment(
                [Pos2::new(rect.max.x, rect.min.y), Pos2::new(rect.max.x, rect.max.y)],
                Stroke::new(1.0, BORDER),
            );

            ui.vertical_centered(|ui| {
                ui.add_space(18.0);
                ui.label(
                    egui::RichText::new("Im")
                        .size(22.0)
                        .color(ACCENT)
                        .strong(),
                );
                ui.add_space(20.0);

                for c in Chapitre::toutes() {
                    let selected = self.chapitre == *c;
                    let resp = nav_bouton(ui, c.glyphe(), c.titre(), selected);
                    if resp.clicked() {
                        self.chapitre = *c;
                    }
                    ui.add_space(6.0);
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    let open = nav_bouton(ui, "+", "Ouvrir", false);
                    if open.clicked() {
                        self.ouvrir_dialogue(ui.ctx());
                    }
                });
            });
        });
    }

    fn ui_inspector(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.painter().rect_filled(rect, 0.0, SURFACE);
            ui.painter().line_segment(
                [Pos2::new(rect.min.x, rect.min.y), Pos2::new(rect.min.x, rect.max.y)],
                Stroke::new(1.0, BORDER),
            );

            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect.shrink(14.0)), |ui| {
                ui.label(
                    egui::RichText::new(self.chapitre.titre())
                        .size(20.0)
                        .color(TEXT)
                        .strong(),
                );
                ui.label(
                    egui::RichText::new(self.chapitre.sous_titre())
                        .size(12.0)
                        .color(TEXT_MUTED),
                );
                ui.add_space(12.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        section(ui, "Opération");
                        for op in self.chapitre.operations() {
                            op_choix(ui, &mut self.operation, *op);
                        }

                        ui.add_space(8.0);
                        self.ui_parametres(ui);

                        if self.operation.besoin_image_b() {
                            ui.add_space(12.0);
                            section(ui, "Image B");
                            let nom_b = self
                                .chemin_b
                                .as_ref()
                                .and_then(|p| p.file_name()?.to_str().map(str::to_owned))
                                .unwrap_or_else(|| "aucune".into());
                            ui.label(
                                egui::RichText::new(nom_b)
                                    .size(12.5)
                                    .color(TEXT_DIM),
                            );
                            ui.add_space(6.0);
                            if ui
                                .add(
                                    egui::Button::new("Charger l'image B…")
                                        .fill(SURFACE_3)
                                        .stroke(Stroke::new(1.0, BORDER))
                                        .corner_radius(8.0)
                                        .min_size(Vec2::new(ui.available_width(), 32.0)),
                                )
                                .clicked()
                            {
                                self.ouvrir_dialogue_b();
                            }
                        }

                        ui.add_space(16.0);
                        let peut = self.peut_appliquer();
                        let apply = ui.add_enabled(
                            peut,
                            egui::Button::new(
                                egui::RichText::new("Appliquer")
                                    .size(14.0)
                                    .color(if peut { BG } else { TEXT_MUTED })
                                    .strong(),
                            )
                            .fill(if peut { ACCENT } else { SURFACE_3 })
                            .corner_radius(10.0)
                            .min_size(Vec2::new(ui.available_width(), 40.0)),
                        );
                        if apply.clicked() {
                            self.appliquer(ui.ctx());
                        }

                        ui.add_space(8.0);
                        if ui
                            .add(
                                egui::Button::new(
                                    egui::RichText::new("Ouvrir image A…")
                                        .size(13.0)
                                        .color(TEXT),
                                )
                                .fill(SURFACE_3)
                                .stroke(Stroke::new(1.0, BORDER))
                                .corner_radius(10.0)
                                .min_size(Vec2::new(ui.available_width(), 36.0)),
                            )
                            .clicked()
                        {
                            self.ouvrir_dialogue(ui.ctx());
                        }

                        if let Some(chemin) = &self.chemin {
                            ui.add_space(14.0);
                            section(ui, "Fichier A");
                            ui.label(
                                egui::RichText::new(
                                    chemin
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("—"),
                                )
                                .size(13.0)
                                .color(TEXT_DIM),
                            );
                        }

                        ui.add_space(16.0);
                        section(ui, "Statistiques");
                        if let Some(s) = &self.stats_entree {
                            stats_ligne(ui, "Entrée", s);
                        } else {
                            ui.label(
                                egui::RichText::new("Aucune image")
                                    .size(12.0)
                                    .color(TEXT_MUTED),
                            );
                        }
                        if let Some(s) = &self.stats_sortie {
                            ui.add_space(6.0);
                            stats_ligne(ui, "Sortie", s);
                        }

                        ui.add_space(16.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(
                                "Branché sur ponctuel & arithmetique\n(tes modules Jeff).",
                            )
                            .size(11.0)
                            .color(TEXT_MUTED),
                        );
                    });
            });
        });
    }

    fn ui_parametres(&mut self, ui: &mut egui::Ui) {
        match self.operation {
            Operation::Saturation => {
                section(ui, &format!("Smin · {:.0}", self.s_min));
                ui.add(
                    egui::Slider::new(&mut self.s_min, 0.0..=254.0)
                        .show_value(false)
                        .trailing_fill(true),
                );
                if self.s_max <= self.s_min {
                    self.s_max = (self.s_min + 1.0).min(255.0);
                }
                section(ui, &format!("Smax · {:.0}", self.s_max));
                ui.add(
                    egui::Slider::new(&mut self.s_max, (self.s_min + 1.0)..=255.0)
                        .show_value(false)
                        .trailing_fill(true),
                );
            }
            Operation::Gamma => {
                section(ui, &format!("γ · {:.2}", self.gamma));
                ui.add(
                    egui::Slider::new(&mut self.gamma, 0.1..=3.0)
                        .show_value(false)
                        .trailing_fill(true),
                );
            }
            Operation::Multiplication => {
                section(ui, &format!("Ratio · {:.2}", self.ratio));
                ui.add(
                    egui::Slider::new(&mut self.ratio, 0.0..=3.0)
                        .show_value(false)
                        .trailing_fill(true),
                );
            }
            Operation::Combinaison => {
                section(ui, &format!("α · {:.2}", self.alpha));
                ui.add(
                    egui::Slider::new(&mut self.alpha, 0.0..=1.0)
                        .show_value(false)
                        .trailing_fill(true),
                );
            }
            _ => {}
        }
    }

    fn ui_main(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.painter().rect_filled(rect, 0.0, BG);

            let pad = 20.0;
            let hist_h = 118.0;
            let header_h = 36.0;
            let gap = 14.0;

            let content = rect.shrink2(Vec2::new(pad, pad));
            let header = Rect::from_min_size(content.min, Vec2::new(content.width(), header_h));
            let hist = Rect::from_min_max(
                Pos2::new(content.min.x, content.max.y - hist_h),
                content.max,
            );
            let canvas = Rect::from_min_max(
                Pos2::new(content.min.x, header.max.y + gap),
                Pos2::new(content.max.x, hist.min.y - gap),
            );

            ui.allocate_new_ui(egui::UiBuilder::new().max_rect(header), |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Studio")
                            .size(15.0)
                            .color(TEXT)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("comparateur avant / après")
                            .size(12.5)
                            .color(TEXT_MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.sortie.is_some() {
                            ui.label(
                                egui::RichText::new(format!("split {:.0}%", self.split * 100.0))
                                    .size(12.0)
                                    .color(TEXT_DIM),
                            );
                        }
                    });
                });
            });

            if self.entree.is_none() {
                self.ui_empty(ui, canvas);
            } else {
                self.ui_comparateur(ui, canvas);
            }

            self.ui_histogrammes(ui, hist);
        });
    }

    fn ui_empty(&mut self, ui: &mut egui::Ui, rect: Rect) {
        let pulse = (self.pulse.sin() * 0.5 + 0.5) * 18.0;
        let border = if self.hover_drop {
            ACCENT
        } else {
            Color32::from_rgb(
                BORDER.r().saturating_add(pulse as u8),
                BORDER.g().saturating_add(pulse as u8),
                BORDER.b().saturating_add((pulse * 0.6) as u8),
            )
        };
        let fill = if self.hover_drop {
            ACCENT_SOFT
        } else {
            SURFACE_2
        };

        ui.painter()
            .rect_filled(rect, CornerRadius::same(14), fill);
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(1.5, border),
            egui::StrokeKind::Inside,
        );

        let resp = ui.allocate_rect(rect, Sense::click());
        if resp.clicked() {
            self.ouvrir_dialogue(ui.ctx());
        }

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(rect.height() * 0.28);
                ui.label(egui::RichText::new("⇩").size(36.0).color(ACCENT));
                ui.add_space(10.0);
                ui.label(
                    egui::RichText::new("Dépose une image ici")
                        .size(22.0)
                        .color(TEXT)
                        .strong(),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new("ou clique pour parcourir · PGM, PPM, PNG, JPEG")
                        .size(13.0)
                        .color(TEXT_MUTED),
                );
                ui.add_space(18.0);
                let _ = ui.add(
                    egui::Button::new(
                        egui::RichText::new("Parcourir les fichiers")
                            .color(BG)
                            .strong(),
                    )
                    .fill(ACCENT)
                    .corner_radius(10.0)
                    .min_size(Vec2::new(200.0, 38.0)),
                );
                // Le click sur le bouton est aussi capté par allocate_rect parent.
            });
        });
        ui.ctx().request_repaint();
    }

    fn ui_comparateur(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(14), Color32::from_rgb(6, 8, 12));
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(1.0, BORDER),
            egui::StrokeKind::Inside,
        );

        let Some(tex_in) = self.tex_entree.clone() else {
            return;
        };
        let tex_out = self.tex_sortie.clone();
        let size = tex_in.size_vec2();
        let fitted = fit_inside(size, rect.shrink(12.0).size());
        let img_rect = Rect::from_center_size(rect.center(), fitted);

        // Entrée (fond)
        egui::Image::new(&tex_in)
            .fit_to_exact_size(fitted)
            .paint_at(ui, img_rect);

        let split_x = img_rect.left() + img_rect.width() * self.anim_split;

        if let Some(tex_out) = tex_out {
            // Sortie clipée à droite du split
            let clip = Rect::from_min_max(
                Pos2::new(split_x, img_rect.top()),
                img_rect.max,
            )
            .intersect(img_rect);
            if clip.width() > 0.5 && clip.height() > 0.5 {
                let mut mesh = Mesh::with_texture(tex_out.id());
                mesh.add_rect_with_uv(
                    clip,
                    Rect::from_min_max(
                        Pos2::new(
                            (clip.min.x - img_rect.min.x) / img_rect.width(),
                            (clip.min.y - img_rect.min.y) / img_rect.height(),
                        ),
                        Pos2::new(
                            (clip.max.x - img_rect.min.x) / img_rect.width(),
                            (clip.max.y - img_rect.min.y) / img_rect.height(),
                        ),
                    ),
                    Color32::WHITE,
                );
                ui.painter().add(Shape::mesh(mesh));
            }

            // Ligne + poignée
            let painter = ui.painter();
            painter.line_segment(
                [
                    Pos2::new(split_x, img_rect.top()),
                    Pos2::new(split_x, img_rect.bottom()),
                ],
                Stroke::new(2.0, ACCENT),
            );
            let handle = Rect::from_center_size(
                Pos2::new(split_x, img_rect.center().y),
                Vec2::new(18.0, 44.0),
            );
            painter.rect_filled(handle, 8.0, ACCENT);
            painter.text(
                handle.center(),
                egui::Align2::CENTER_CENTER,
                "⇄",
                egui::FontId::proportional(14.0),
                BG,
            );

            // Labels
            painter.text(
                Pos2::new(img_rect.left() + 10.0, img_rect.top() + 10.0),
                egui::Align2::LEFT_TOP,
                "AVANT",
                egui::FontId::proportional(11.0),
                TEXT_DIM,
            );
            painter.text(
                Pos2::new(img_rect.right() - 10.0, img_rect.top() + 10.0),
                egui::Align2::RIGHT_TOP,
                "APRÈS",
                egui::FontId::proportional(11.0),
                TEXT_DIM,
            );

            let resp = ui.allocate_rect(img_rect, Sense::click_and_drag());
            if resp.dragged() || resp.is_pointer_button_down_on() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    self.split = ((pos.x - img_rect.left()) / img_rect.width()).clamp(0.02, 0.98);
                }
            }
        } else {
            ui.painter().text(
                Pos2::new(img_rect.center().x, img_rect.bottom() + 8.0).max(Pos2::new(
                    img_rect.center().x,
                    rect.bottom() - 18.0,
                )),
                egui::Align2::CENTER_BOTTOM,
                "Applique une opération pour comparer",
                egui::FontId::proportional(12.0),
                TEXT_MUTED,
            );
        }
    }

    fn ui_histogrammes(&mut self, ui: &mut egui::Ui, rect: Rect) {
        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
            theme::carte().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("Histogrammes")
                            .size(13.0)
                            .color(TEXT)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new("distribution des niveaux de gris")
                            .size(11.5)
                            .color(TEXT_MUTED),
                    );
                });
                ui.add_space(8.0);
                let h = ui.available_height().max(56.0);
                let w = ui.available_width();
                let demi = (w - 12.0) / 2.0;
                ui.horizontal(|ui| {
                    dessiner_histogramme(
                        ui,
                        "Entrée",
                        &self.hist_entree,
                        self.entree.is_some(),
                        Vec2::new(demi, h),
                    );
                    ui.add_space(12.0);
                    dessiner_histogramme(
                        ui,
                        "Sortie",
                        &self.hist_sortie,
                        self.sortie.is_some(),
                        Vec2::new(demi, h),
                    );
                });
            });
        });
    }

    fn ui_toast(&mut self, ui: &mut egui::Ui, full: Rect) {
        let Some(toast) = &self.toast else {
            return;
        };
        let age = toast.cree.elapsed();
        if age > Duration::from_secs(3) {
            self.toast = None;
            return;
        }
        let fade = if age < Duration::from_millis(200) {
            age.as_secs_f32() / 0.2
        } else if age > Duration::from_millis(2600) {
            1.0 - (age.as_secs_f32() - 2.6) / 0.4
        } else {
            1.0
        };

        let color = if toast.ok { SUCCESS } else { DANGER };
        let bg = Color32::from_rgba_unmultiplied(24, 28, 36, (230.0 * fade) as u8);
        let size = Vec2::new(360.0, 48.0);
        let pos = Pos2::new(full.center().x - size.x / 2.0, full.max.y - 80.0);
        let toast_rect = Rect::from_min_size(pos, size);

        ui.painter()
            .rect_filled(toast_rect, 12.0, bg);
        ui.painter().rect_stroke(
            toast_rect,
            12.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), (200.0 * fade) as u8)),
            egui::StrokeKind::Inside,
        );
        ui.painter().text(
            toast_rect.center(),
            egui::Align2::CENTER_CENTER,
            &toast.texte,
            egui::FontId::proportional(13.0),
            Color32::from_rgba_unmultiplied(TEXT.r(), TEXT.g(), TEXT.b(), (255.0 * fade) as u8),
        );
    }
}

fn nav_bouton(ui: &mut egui::Ui, glyphe: &str, tip: &str, selected: bool) -> egui::Response {
    let size = Vec2::new(48.0, 48.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let bg = if selected {
        ACCENT_SOFT
    } else if hovered {
        SURFACE_3
    } else {
        Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, 12.0, bg);
    if selected {
        ui.painter().rect_stroke(
            rect,
            12.0,
            Stroke::new(1.0, ACCENT),
            egui::StrokeKind::Inside,
        );
    }
    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        glyphe,
        egui::FontId::proportional(18.0),
        if selected || hovered { ACCENT } else { TEXT_DIM },
    );
    resp.on_hover_text(tip)
}

fn section(ui: &mut egui::Ui, titre: &str) {
    ui.label(
        egui::RichText::new(titre.to_uppercase())
            .size(11.0)
            .color(TEXT_MUTED)
            .strong(),
    );
    ui.add_space(6.0);
}

fn op_choix(ui: &mut egui::Ui, courant: &mut Operation, valeur: Operation) {
    let selected = *courant == valeur;
    let fill = if selected { ACCENT_SOFT } else { SURFACE_3 };
    let stroke = if selected {
        Stroke::new(1.0, ACCENT)
    } else {
        Stroke::new(1.0, BORDER)
    };
    let resp = ui.add(
        egui::Button::new(
            egui::RichText::new(valeur.label())
                .size(13.0)
                .color(if selected { ACCENT } else { TEXT }),
        )
        .fill(fill)
        .stroke(stroke)
        .corner_radius(8.0)
        .min_size(Vec2::new(ui.available_width(), 34.0)),
    );
    if resp.clicked() {
        *courant = valeur;
    }
    ui.add_space(6.0);
}

fn stats_ligne(ui: &mut egui::Ui, label: &str, s: &Stats) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(label).size(12.0).color(TEXT_DIM));
        ui.label(
            egui::RichText::new(format!(
                "min {} · max {} · μ {:.1}",
                s.min, s.max, s.moyenne
            ))
            .size(12.0)
            .color(TEXT),
        );
    });
}

fn dessiner_histogramme(
    ui: &mut egui::Ui,
    titre: &str,
    hist: &[u32; 256],
    actif: bool,
    taille: Vec2,
) {
    ui.vertical(|ui| {
        ui.set_width(taille.x);
        ui.label(egui::RichText::new(titre).size(11.0).color(TEXT_MUTED));
        let (rect, _) = ui.allocate_exact_size(Vec2::new(taille.x, taille.y - 18.0), Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 8.0, Color32::from_rgb(10, 12, 16));

        if !actif {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "—",
                egui::FontId::proportional(14.0),
                TEXT_MUTED,
            );
            return;
        }

        let max = hist.iter().copied().max().unwrap_or(1).max(1) as f32;
        let w = rect.width() / 256.0;
        for (i, &c) in hist.iter().enumerate() {
            let h = (c as f32 / max) * (rect.height() - 2.0);
            if h < 0.5 {
                continue;
            }
            let t = i as f32 / 255.0;
            let color = lerp_color(HIST_BOT, HIST_TOP, t);
            let x = rect.left() + i as f32 * w;
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, rect.bottom() - h),
                    Pos2::new(x + w.max(1.0), rect.bottom()),
                ),
                0.0,
                color,
            );
        }
    });
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgb(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
    )
}

fn fit_inside(img: Vec2, box_size: Vec2) -> Vec2 {
    let sx = box_size.x / img.x;
    let sy = box_size.y / img.y;
    let s = sx.min(sy);
    img * s
}

fn stats_depuis_lib(img: &ImageGris) -> Stats {
    let (min, max) = ponctuel::dynamique(img);
    Stats {
        min,
        max,
        moyenne: ponctuel::luminance(img) as f32,
    }
}

fn texture_depuis_gris(ctx: &egui::Context, nom: &str, img: &ImageGris) -> TextureHandle {
    let (l, h) = img.dimensions();
    let mut rgba = Vec::with_capacity(l * h * 4);
    for &p in img.donnees() {
        rgba.extend_from_slice(&[p, p, p, 255]);
    }
    let color = ColorImage::from_rgba_unmultiplied([l, h], &rgba);
    ctx.load_texture(nom, color, egui::TextureOptions::LINEAR)
}

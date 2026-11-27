use clap::Subcommand;
use imagerie::es;
use imagerie::interpolation::{self, Methode};
use imagerie::Resultat;

/// Convertit la chaîne "ppv", "bilineaire" ou "bicubique" en `Methode`.
fn parse_methode(s: &str) -> Methode {
    match s {
        "ppv" => Methode::PlusProcheVoisin,
        "bicubique" => Methode::Bicubique,
        _ => Methode::Bilineaire,
    }
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Zoom d'un facteur réel (> 1 agrandit, < 1 réduit).
    Zoom {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 2.0)]
        facteur: f64,
        /// ppv, bilineaire ou bicubique.
        #[arg(short, long, default_value = "bilineaire")]
        methode: String,
    },

    /// Redimensionne à une taille exacte (largeur × hauteur).
    Redimensionner {
        entree: String,
        sortie: String,
        largeur: usize,
        hauteur: usize,
        /// ppv, bilineaire ou bicubique.
        #[arg(short, long, default_value = "bilineaire")]
        methode: String,
    },

    /// Zoom couleur d'un facteur réel.
    ZoomCouleur {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 2.0)]
        facteur: f64,
        /// ppv, bilineaire ou bicubique.
        #[arg(short, long, default_value = "bilineaire")]
        methode: String,
    },

    /// Agrandissement par copie des lignes et colonnes (facteur entier).
    Copie {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 2)]
        facteur: usize,
    },

    /// Zoom x2 séparable (lignes puis colonnes).
    ZoomSeparable {
        entree: String,
        sortie: String,
        /// ppv, bilineaire ou bicubique.
        #[arg(short, long, default_value = "bilineaire")]
        methode: String,
    },

    /// Rotation d'un angle en degrés autour du centre.
    Rotation {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 45.0)]
        angle: f64,
        /// ppv, bilineaire ou bicubique.
        #[arg(short, long, default_value = "bilineaire")]
        methode: String,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Zoom {
            entree,
            sortie,
            facteur,
            methode,
        } => {
            let img = es::charger_gris(entree)?;
            let r = interpolation::zoom(&img, facteur, parse_methode(&methode))?;
            es::enregistrer_gris(&r, sortie)
        }

        Cmd::Redimensionner {
            entree,
            sortie,
            largeur,
            hauteur,
            methode,
        } => {
            let img = es::charger_gris(entree)?;
            let m = parse_methode(&methode);
            let r = interpolation::redimensionner(&img, largeur, hauteur, m)?;
            es::enregistrer_gris(&r, sortie)
        }

        Cmd::ZoomCouleur {
            entree,
            sortie,
            facteur,
            methode,
        } => {
            let img = es::charger_couleur(entree)?;
            let r = interpolation::zoom_rvb(&img, facteur, parse_methode(&methode))?;
            es::enregistrer_couleur(&r, sortie)
        }

        Cmd::Copie {
            entree,
            sortie,
            facteur,
        } => {
            let img = es::charger_gris(entree)?;
            let r = interpolation::agrandir_par_copie(&img, facteur)?;
            es::enregistrer_gris(&r, sortie)
        }

        Cmd::ZoomSeparable {
            entree,
            sortie,
            methode,
        } => {
            let img = es::charger_gris(entree)?;
            let r = interpolation::zoom_x2_separable(&img, parse_methode(&methode));
            es::enregistrer_gris(&r, sortie)
        }

        Cmd::Rotation {
            entree,
            sortie,
            angle,
            methode,
        } => {
            let img = es::charger_gris(entree)?;
            let r = interpolation::rotation(&img, angle, parse_methode(&methode));
            es::enregistrer_gris(&r, sortie)
        }
    }
}

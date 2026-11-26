use clap::{Subcommand, ValueEnum};
use imagerie::contours::{self, Laplacien, Operateur};
use imagerie::es;
use imagerie::{Bord, Resultat};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum OperateurArg {
    DeriveeSimple,
    DeriveeCentree,
    Roberts,
    Prewitt,
    Sobel,
}

impl From<OperateurArg> for Operateur {
    fn from(op: OperateurArg) -> Operateur {
        match op {
            OperateurArg::DeriveeSimple => Operateur::DeriveeSimple,
            OperateurArg::DeriveeCentree => Operateur::DeriveeCentree,
            OperateurArg::Roberts => Operateur::Roberts,
            OperateurArg::Prewitt => Operateur::Prewitt,
            OperateurArg::Sobel => Operateur::Sobel,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum LaplacienArg {
    Quatre,
    Huit,
}

impl From<LaplacienArg> for Laplacien {
    fn from(l: LaplacienArg) -> Laplacien {
        match l {
            LaplacienArg::Quatre => Laplacien::Quatre,
            LaplacienArg::Huit => Laplacien::Huit,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum BordArg {
    Zero,
    Miroir,
    Replique,
    Partiel,
}

impl From<BordArg> for Bord {
    fn from(b: BordArg) -> Bord {
        match b {
            BordArg::Zero => Bord::Zero,
            BordArg::Miroir => Bord::Miroir,
            BordArg::Replique => Bord::Replique,
            BordArg::Partiel => Bord::Partiel,
        }
    }
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Contours par seuillage de la norme du gradient de Sobel.
    Sobel {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 60.0)]
        seuil: f64,
    },
    /// Norme du gradient (exacte ou approchée) pour un opérateur donné.
    Gradient {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = OperateurArg::Sobel)]
        operateur: OperateurArg,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
        /// Utilise la norme approchée |Gx| + |Gy| au lieu de sqrt(Gx² + Gy²).
        #[arg(long)]
        approche: bool,
    },
    /// Contours binaires par seuillage de la norme du gradient.
    ContoursGradient {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = OperateurArg::Sobel)]
        operateur: OperateurArg,
        #[arg(short, long, default_value_t = 60.0)]
        seuil: f64,
    },
    /// Laplacien (4 ou 8 connexité) normalisé pour affichage.
    Laplacien {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = LaplacienArg::Quatre)]
        variante: LaplacienArg,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Passages par zéro du laplacien.
    PassagesZero {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = LaplacienArg::Quatre)]
        variante: LaplacienArg,
        #[arg(short, long, default_value_t = 20.0)]
        seuil: f64,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Amincissement des contours par suppression des non-maxima.
    SuppressionNonMaxima {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = OperateurArg::Sobel)]
        operateur: OperateurArg,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Sobel {
            entree,
            sortie,
            seuil,
        } => {
            let r =
                contours::contours_gradient(&es::charger_gris(entree)?, Operateur::Sobel, seuil);
            es::enregistrer_gris(&r.vers_gris(), sortie)
        }
        Cmd::Gradient {
            entree,
            sortie,
            operateur,
            bord,
            approche,
        } => {
            let g = contours::gradient(&es::charger_gris(entree)?, operateur.into(), bord.into());
            let norme = if approche {
                g.norme_approchee()
            } else {
                g.norme()
            };
            es::enregistrer_gris(&norme.vers_gris_normalise(), sortie)
        }
        Cmd::ContoursGradient {
            entree,
            sortie,
            operateur,
            seuil,
        } => {
            let r =
                contours::contours_gradient(&es::charger_gris(entree)?, operateur.into(), seuil);
            es::enregistrer_gris(&r.vers_gris(), sortie)
        }
        Cmd::Laplacien {
            entree,
            sortie,
            variante,
            bord,
        } => {
            let r = contours::laplacien(&es::charger_gris(entree)?, variante.into(), bord.into());
            es::enregistrer_gris(&r.vers_gris_normalise(), sortie)
        }
        Cmd::PassagesZero {
            entree,
            sortie,
            variante,
            seuil,
            bord,
        } => {
            let lap = contours::laplacien(&es::charger_gris(entree)?, variante.into(), bord.into());
            let r = contours::passages_par_zero(&lap, seuil);
            es::enregistrer_gris(&r.vers_gris(), sortie)
        }
        Cmd::SuppressionNonMaxima {
            entree,
            sortie,
            operateur,
            bord,
        } => {
            let g = contours::gradient(&es::charger_gris(entree)?, operateur.into(), bord.into());
            let r = contours::suppression_non_maxima(&g);
            es::enregistrer_gris(&r.vers_gris_normalise(), sortie)
        }
    }
}

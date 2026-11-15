use clap::{Subcommand, ValueEnum};
use imagerie::es;
use imagerie::filtrage;
use imagerie::Bord;
use imagerie::Resultat;

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum BordArg {
    Zero,
    Miroir,
    Replique,
    Partiel, // ← était "Partielle"
}

impl From<BordArg> for Bord {
    fn from(b: BordArg) -> Bord {
        match b {
            BordArg::Zero => Bord::Zero,
            BordArg::Miroir => Bord::Miroir,
            BordArg::Replique => Bord::Replique,
            BordArg::Partiel => Bord::Partiel, // ← était "Bord::Partielle"
        }
    }
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Filtre médian n x n.
    Median {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 3)]
        taille: usize,
        #[arg(long, value_enum, default_value_t = BordArg::Miroir)]
        bord: BordArg,
    },
    /// Filtre moyenneur n x n.
    Moyenneur {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 3)]
        taille: usize,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Filtre gaussien n x n d'écart-type sigma.
    Gaussien {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 5)]
        taille: usize,
        #[arg(short, long, default_value_t = 1.0)]
        sigma: f64,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Filtre rehausseur (passe-haut).
    Rehausseur {
        entree: String,
        sortie: String,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Filtre minimum n x n.
    Min {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 3)]
        taille: usize,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Filtre maximum n x n.
    Max {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 3)]
        taille: usize,
        #[arg(long, value_enum, default_value_t = BordArg::Replique)]
        bord: BordArg,
    },
    /// Bruit poivre et sel.
    BruitPoivreSel {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 0.05)]
        proportion: f64,
        #[arg(short, long, default_value_t = 42)]
        graine: u64,
    },
    /// Bruit gaussien additif.
    BruitGaussien {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 10.0)]
        sigma: f64,
        #[arg(short, long, default_value_t = 42)]
        graine: u64,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Median {
            entree,
            sortie,
            taille,
            bord,
        } => {
            let r = filtrage::filtre_median(&es::charger_gris(entree)?, taille, bord.into())?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::Moyenneur {
            entree,
            sortie,
            taille,
            bord,
        } => {
            let r = filtrage::filtre_moyenneur(&es::charger_gris(entree)?, taille, bord.into())?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::Gaussien {
            entree,
            sortie,
            taille,
            sigma,
            bord,
        } => {
            let r =
                filtrage::filtre_gaussien(&es::charger_gris(entree)?, taille, sigma, bord.into())?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::Rehausseur {
            entree,
            sortie,
            bord,
        } => {
            let r = filtrage::filtre_rehausseur(&es::charger_gris(entree)?, bord.into());
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::Min {
            entree,
            sortie,
            taille,
            bord,
        } => {
            let r = filtrage::filtre_min(&es::charger_gris(entree)?, taille, bord.into())?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::Max {
            entree,
            sortie,
            taille,
            bord,
        } => {
            let r = filtrage::filtre_max(&es::charger_gris(entree)?, taille, bord.into())?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::BruitPoivreSel {
            entree,
            sortie,
            proportion,
            graine,
        } => {
            let r = filtrage::bruit_poivre_sel(&es::charger_gris(entree)?, proportion, graine)?;
            es::enregistrer_gris(&r, sortie)
        }
        Cmd::BruitGaussien {
            entree,
            sortie,
            sigma,
            graine,
        } => {
            let r = filtrage::bruit_gaussien(&es::charger_gris(entree)?, sigma, graine);
            es::enregistrer_gris(&r, sortie)
        }
    }
}

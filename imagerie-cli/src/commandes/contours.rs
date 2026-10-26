use clap::Subcommand;
use imagerie::contours::{self, Operateur};
use imagerie::es;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Contours par seuillage de la norme du gradient de Sobel.
    Sobel {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 60.0)]
        seuil: f64,
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
    }
}

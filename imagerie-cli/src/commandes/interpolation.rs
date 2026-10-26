use clap::Subcommand;
use imagerie::es;
use imagerie::interpolation::{self, Methode};
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Zoom d'un facteur réel.
    Zoom {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 2.0)]
        facteur: f64,
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
            let methode = match methode.as_str() {
                "ppv" => Methode::PlusProcheVoisin,
                "bicubique" => Methode::Bicubique,
                _ => Methode::Bilineaire,
            };
            let r = interpolation::zoom(&es::charger_gris(entree)?, facteur, methode)?;
            es::enregistrer_gris(&r, sortie)
        }
    }
}

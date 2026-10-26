use clap::Subcommand;
use imagerie::bases;
use imagerie::es;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Conversion couleur -> niveaux de gris (luminance).
    Gris { entree: String, sortie: String },
    /// Réduction du nombre de niveaux de gris.
    Quantifier {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 8)]
        niveaux: u16,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Gris { entree, sortie } => {
            es::enregistrer_gris(&bases::vers_gris(&es::charger_couleur(entree)?), sortie)
        }
        Cmd::Quantifier {
            entree,
            sortie,
            niveaux,
        } => es::enregistrer_gris(
            &bases::quantifier(&es::charger_gris(entree)?, niveaux)?,
            sortie,
        ),
    }
}

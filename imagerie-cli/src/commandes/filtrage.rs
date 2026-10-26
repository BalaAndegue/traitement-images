use clap::Subcommand;
use imagerie::es;
use imagerie::filtrage;
use imagerie::Bord;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Filtre médian n x n.
    Median {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 3)]
        taille: usize,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Median {
            entree,
            sortie,
            taille,
        } => {
            let r = filtrage::filtre_median(&es::charger_gris(entree)?, taille, Bord::Miroir)?;
            es::enregistrer_gris(&r, sortie)
        }
    }
}

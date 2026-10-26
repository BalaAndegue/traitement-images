use clap::Subcommand;
use imagerie::binaire;
use imagerie::es;
use imagerie::Connexite;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Étiquetage des composantes connexes (image d'entrée : objets clairs sur fond noir).
    Etiquetage {
        entree: String,
        sortie: String,
        /// Utiliser la 8-connexité au lieu de la 4-connexité.
        #[arg(long)]
        huit: bool,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Etiquetage {
            entree,
            sortie,
            huit,
        } => {
            let masque = es::charger_gris(entree)?.map(|&p| p >= 128);
            let connexite = if huit {
                Connexite::Huit
            } else {
                Connexite::Quatre
            };
            let (etiquettes, n) = binaire::etiqueter(&masque, connexite);
            println!("{n} composantes connexes");
            es::enregistrer_couleur(&etiquettes.vers_couleurs(), sortie)
        }
    }
}

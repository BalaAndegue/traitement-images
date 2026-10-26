//! Application en ligne de commande.
//!
//! Exemple : `imagerie ponctuel egalisation images/mire.pgm sortie.pgm`
//!
//! Chaque module du cours a son fichier dans `commandes/` : on y ajoute une
//! variante par fonction de la bibliothèque, sans toucher aux fichiers des autres.

mod commandes;

use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "imagerie",
    version,
    about = "Traitement d'images (cours du Dr Tapamo)"
)]
struct Cli {
    #[command(subcommand)]
    commande: Commande,
}

#[derive(Subcommand)]
enum Commande {
    /// Affiche la taille et le type d'une image.
    Info { entree: String },
    /// Convertit une image d'un format à l'autre (d'après l'extension de sortie).
    Convertir {
        entree: String,
        sortie: String,
        /// Force la conversion en niveaux de gris.
        #[arg(long)]
        gris: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let resultat = match cli.commande {
        Commande::Info { entree } => commandes::info(&entree),
        Commande::Convertir {
            entree,
            sortie,
            gris,
        } => commandes::convertir(&entree, &sortie, gris),
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

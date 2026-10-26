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
    /// Chapitre 1 : canaux, quantification, échantillonnage, profils.
    #[command(subcommand)]
    Bases(commandes::bases::Cmd),
    /// Chapitre 2 : histogramme, contraste, égalisation.
    #[command(subcommand)]
    Ponctuel(commandes::ponctuel::Cmd),
    /// Chapitre 2 : opérations logiques et arithmétiques.
    #[command(subcommand)]
    Arith(commandes::arithmetique::Cmd),
    /// Chapitre 2 : interpolation et changement d'échelle.
    #[command(subcommand)]
    Interpolation(commandes::interpolation::Cmd),
    /// Chapitre 3 : convolution et filtres.
    #[command(subcommand)]
    Filtre(commandes::filtrage::Cmd),
    /// Chapitre 4 : transformée de Fourier.
    #[command(subcommand)]
    Fourier(commandes::fourier::Cmd),
    /// Chapitre 5 : contours et transformée de Hough.
    #[command(subcommand)]
    Contours(commandes::contours::Cmd),
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
        Commande::Bases(c) => commandes::bases::executer(c),
        Commande::Ponctuel(c) => commandes::ponctuel::executer(c),
        Commande::Arith(c) => commandes::arithmetique::executer(c),
        Commande::Interpolation(c) => commandes::interpolation::executer(c),
        Commande::Filtre(c) => commandes::filtrage::executer(c),
        Commande::Fourier(c) => commandes::fourier::executer(c),
        Commande::Contours(c) => commandes::contours::executer(c),
    };
    match resultat {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erreur : {e}");
            ExitCode::FAILURE
        }
    }
}

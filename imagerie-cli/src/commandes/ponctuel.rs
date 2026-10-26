use clap::Subcommand;
use imagerie::es;
use imagerie::ponctuel;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Égalisation d'histogramme.
    Egalisation { entree: String, sortie: String },
    /// Luminance, contraste et dynamique.
    Stats { entree: String },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Egalisation { entree, sortie } => es::enregistrer_gris(
            &ponctuel::egalisation_histogramme(&es::charger_gris(entree)?),
            sortie,
        ),
        Cmd::Stats { entree } => {
            let img = es::charger_gris(entree)?;
            let (min, max) = ponctuel::dynamique(&img);
            println!("dynamique : [{min}, {max}]");
            println!("luminance : {:.2}", ponctuel::luminance(&img));
            println!(
                "contraste (écart-type) : {:.2}",
                ponctuel::contraste_ecart_type(&img)
            );
            Ok(())
        }
    }
}

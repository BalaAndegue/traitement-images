use clap::Subcommand;
use imagerie::arithmetique;
use imagerie::es;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Addition saturée de deux images.
    Addition {
        a: String,
        b: String,
        sortie: String,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Addition { a, b, sortie } => {
            let r = arithmetique::addition(&es::charger_gris(a)?, &es::charger_gris(b)?)?;
            es::enregistrer_gris(&r, sortie)
        }
    }
}

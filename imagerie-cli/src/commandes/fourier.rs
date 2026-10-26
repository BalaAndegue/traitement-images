use clap::Subcommand;
use imagerie::es;
use imagerie::fourier;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Spectre de Fourier rehaussé log(1 + |F|).
    Spectre { entree: String, sortie: String },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Spectre { entree, sortie } => {
            let spectre = fourier::fft_2d(&es::charger_gris(entree)?);
            es::enregistrer_gris(&fourier::spectre_affichable(&spectre), sortie)
        }
    }
}

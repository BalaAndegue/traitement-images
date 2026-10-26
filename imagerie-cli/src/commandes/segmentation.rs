use clap::Subcommand;
use imagerie::es;
use imagerie::segmentation;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    /// Seuillage automatique par la méthode d'Otsu.
    Otsu { entree: String, sortie: String },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Otsu { entree, sortie } => {
            let img = es::charger_gris(entree)?;
            let t = segmentation::seuil_otsu(&img);
            println!("seuil d'Otsu : {t}");
            es::enregistrer_gris(&segmentation::seuillage(&img, t).vers_gris(), sortie)
        }
    }
}

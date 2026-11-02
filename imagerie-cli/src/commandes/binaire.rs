use clap::Subcommand;
use imagerie::binaire;
use imagerie::es;
use imagerie::Connexite;
use imagerie::Resultat;

#[derive(Subcommand)]
pub enum Cmd {
    Etiquetage { entree: String, sortie: String, #[arg(long)] huit: bool },

    /// Nombre d'objets.
    Compter { entree: String, #[arg(long)] huit: bool },

    /// Supprime les objets de moins de `surface_min` pixels.
    Nettoyer {
        entree: String,
        sortie: String,
        surface_min: usize,
        #[arg(long)] huit: bool,
    },

    /// Carte des distances (D4 par défaut, D8 avec --d8).
    Distances { entree: String, sortie: String, #[arg(long)] d8: bool },

    /// Code de Freeman du contour.
    Freeman { entree: String, #[arg(long)] huit: bool },

    /// Seuillage par hystérésis.
    Hysteresis { entree: String, sortie: String, bas: f64, haut: f64 },
}

fn connexite(huit: bool) -> Connexite {
    if huit { Connexite::Huit } else { Connexite::Quatre }
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

        Cmd::Compter { entree, huit } => {
            let masque = es::charger_gris(entree)?.map(|&p| p >= 128);
            println!("{}", binaire::compter_objets(&masque, connexite(huit)));
            Ok(())
        }

        Cmd::Nettoyer { entree, sortie, surface_min, huit } => {
            let masque = es::charger_gris(entree)?.map(|&p| p >= 128);
            let net = binaire::supprimer_petits_objets(&masque, surface_min, connexite(huit));
            es::enregistrer_gris(&net.vers_gris(), sortie)
        }

        Cmd::Distances { entree, sortie, d8 } => {
            let masque = es::charger_gris(entree)?.map(|&p| p >= 128);
            let d = if d8 { binaire::Distance::D8 } else { binaire::Distance::D4 };
            let carte = binaire::carte_distances(&masque, d);
            es::enregistrer_gris(&carte.map(|&v| v as f64).vers_gris_normalise(), sortie)
        }

        Cmd::Freeman { entree, huit } => {
            let masque = es::charger_gris(entree)?.map(|&p| p >= 128);
            match binaire::freeman(&masque, connexite(huit)) {
                Some(c) => println!("départ {:?} : {:?}", c.depart, c.codes),
                None => println!("image vide"),
            }
            Ok(())
        }

        Cmd::Hysteresis { entree, sortie, bas, haut } => {
            let img = es::charger_gris(entree)?.vers_reelle();
            let res = binaire::seuillage_hysteresis(&img, bas, haut)?;
            es::enregistrer_gris(&res.vers_gris(), sortie)
        }
    }
}

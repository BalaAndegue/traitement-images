use clap::{Args, Subcommand};
use imagerie::es;
use imagerie::hough::{self, Accumulateur};
use imagerie::morphologie::{self, ElementStructurant};
use imagerie::segmentation;
use imagerie::{Erreur, ImageBinaire, Resultat, Rvb};

#[derive(Args)]
pub struct OptionsHough {
    /// Image d'entrée : les pixels >= 128 sont les points de contour.
    entree: String,
    /// Pas de theta en degrés.
    #[arg(long, default_value_t = 1.0)]
    pas_theta: f64,
    /// Pas de rho en pixels.
    #[arg(long, default_value_t = 1.0)]
    pas_rho: f64,
    /// Image en niveaux de gris quelconque : on prend le contour intérieur des
    /// objets après un seuillage d'Otsu (en attendant les contours du chapitre 5).
    #[arg(long)]
    contours: bool,
}

#[derive(Args)]
pub struct OptionsPics {
    /// Nombre maximal de droites.
    #[arg(short, long, default_value_t = 5)]
    nb: usize,
    /// Votes minimaux d'un pic.
    #[arg(long, default_value_t = 10)]
    votes_min: u32,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Image de test du cours : points alignés et/ou points aléatoires.
    Points {
        sortie: String,
        #[arg(long, default_value_t = 20)]
        alignes: usize,
        #[arg(long, default_value_t = 0)]
        aleatoires: usize,
        #[arg(long, default_value_t = 200)]
        largeur: usize,
        #[arg(long, default_value_t = 200)]
        hauteur: usize,
        #[arg(long, default_value_t = 1)]
        graine: u64,
    },
    /// Image de l'accumulateur (x = theta de -90° à 90°, y = rho).
    Accumulateur {
        #[command(flatten)]
        hough: OptionsHough,
        sortie: String,
    },
    /// Affiche les droites détectées (pics de l'accumulateur).
    Pics {
        #[command(flatten)]
        hough: OptionsHough,
        #[command(flatten)]
        pics: OptionsPics,
    },
    /// Trace en rouge les droites détectées sur l'image.
    Droites {
        #[command(flatten)]
        hough: OptionsHough,
        sortie: String,
        #[command(flatten)]
        pics: OptionsPics,
    },
}

fn transformer(o: &OptionsHough) -> Resultat<(imagerie::ImageGris, Accumulateur)> {
    for (nom, pas) in [("--pas-theta", o.pas_theta), ("--pas-rho", o.pas_rho)] {
        if !pas.is_finite() || pas < 0.01 {
            return Err(Erreur::Parametre(format!(
                "{nom} doit être un réel >= 0,01 (reçu {pas})"
            )));
        }
    }
    let img = es::charger_gris(&o.entree)?;
    let points: ImageBinaire = if o.contours {
        let objets = segmentation::seuillage(&img, segmentation::seuil_otsu(&img));
        morphologie::gradient_interne(&objets, &ElementStructurant::croix())
    } else {
        img.map(|&p| p >= 128)
    };
    println!("{} points de contour", points.nb_pixels_objet());
    let acc = hough::hough_droites(&points, o.pas_theta, o.pas_rho);
    Ok((img, acc))
}

fn afficher(droites: &[hough::Droite]) {
    for d in droites {
        println!(
            "rho = {:7.1}  theta = {:6.1}°  votes = {}",
            d.rho,
            d.theta.to_degrees(),
            d.votes
        );
    }
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Points {
            sortie,
            alignes,
            aleatoires,
            largeur,
            hauteur,
            graine,
        } => {
            let img = hough::image_points(largeur, hauteur, alignes, aleatoires, graine)?;
            es::enregistrer_gris(&img.vers_gris(), sortie)
        }
        Cmd::Accumulateur { hough, sortie } => {
            let (_, acc) = transformer(&hough)?;
            let (l, h) = acc.votes.dimensions();
            let max = acc.votes.donnees().iter().max().copied().unwrap_or(0);
            println!("accumulateur {l} x {h}, maximum {max} votes");
            es::enregistrer_gris(&acc.affichable(), sortie)
        }
        Cmd::Pics { hough, pics } => {
            let (_, acc) = transformer(&hough)?;
            afficher(&acc.pics(pics.nb, pics.votes_min));
            Ok(())
        }
        Cmd::Droites {
            hough,
            sortie,
            pics,
        } => {
            let (img, acc) = transformer(&hough)?;
            let droites = acc.pics(pics.nb, pics.votes_min);
            afficher(&droites);
            let trace = hough::tracer_droites(&img, &droites, Rvb::new(255, 0, 0));
            es::enregistrer_couleur(&trace, sortie)
        }
    }
}

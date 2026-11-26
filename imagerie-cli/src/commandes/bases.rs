use clap::{Subcommand, ValueEnum};
use imagerie::bases::{self, Canal};
use imagerie::es;
use imagerie::Resultat;

#[derive(Clone, Copy, ValueEnum)]
pub enum NomCanal {
    Rouge,
    Vert,
    Bleu,
}

impl NomCanal {
    fn vers_canal(self) -> Canal {
        match self {
            NomCanal::Rouge => Canal::Rouge,
            NomCanal::Vert => Canal::Vert,
            NomCanal::Bleu => Canal::Bleu,
        }
    }
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Conversion couleur -> niveaux de gris (luminance).
    Gris { entree: String, sortie: String },
    /// Conversion couleur -> niveaux de gris par moyenne (R + V + B) / 3.
    GrisMoyenne { entree: String, sortie: String },
    /// Extrait un canal (rouge, vert ou bleu) en niveaux de gris.
    Canal {
        entree: String,
        sortie: String,
        #[arg(value_enum)]
        canal: NomCanal,
    },
    /// Isole un canal : les deux autres sont mis à zéro.
    Isoler {
        entree: String,
        sortie: String,
        #[arg(value_enum)]
        canal: NomCanal,
    },
    /// Réduction du nombre de niveaux de gris (L = 2^K).
    Quantifier {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 8)]
        niveaux: u16,
    },
    /// Échantillonnage : un pixel sur `pas`, image plus petite.
    Echantillonner {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 2)]
        pas: usize,
    },
    /// Pixelisation : blocs de `pas` x `pas`, taille d'origine conservée.
    Pixeliser {
        entree: String,
        sortie: String,
        #[arg(short, long, default_value_t = 4)]
        pas: usize,
    },
    /// Taille de stockage b = M x N x K.
    Taille {
        entree: String,
        /// Bits par pixel K (8 pour une image grise, 24 pour une image couleur).
        #[arg(short, long, default_value_t = 8)]
        bits: u32,
    },
    /// Profil d'intensité d'une ligne, enregistré comme une courbe.
    ProfilLigne {
        entree: String,
        sortie: String,
        #[arg(short, long)]
        y: usize,
    },
    /// Profil d'intensité d'une colonne, enregistré comme une courbe.
    ProfilColonne {
        entree: String,
        sortie: String,
        #[arg(short, long)]
        x: usize,
    },
    /// Profil d'intensité le long d'un segment (Bresenham).
    ProfilSegment {
        entree: String,
        sortie: String,
        x0: usize,
        y0: usize,
        x1: usize,
        y1: usize,
    },
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Gris { entree, sortie } => {
            es::enregistrer_gris(&bases::vers_gris(&es::charger_couleur(entree)?), sortie)
        }
        Cmd::GrisMoyenne { entree, sortie } => es::enregistrer_gris(
            &bases::vers_gris_moyenne(&es::charger_couleur(entree)?),
            sortie,
        ),
        Cmd::Canal {
            entree,
            sortie,
            canal,
        } => es::enregistrer_gris(
            &bases::extraire_canal(&es::charger_couleur(entree)?, canal.vers_canal()),
            sortie,
        ),
        Cmd::Isoler {
            entree,
            sortie,
            canal,
        } => es::enregistrer_couleur(
            &bases::isoler_canal(&es::charger_couleur(entree)?, canal.vers_canal()),
            sortie,
        ),
        Cmd::Quantifier {
            entree,
            sortie,
            niveaux,
        } => es::enregistrer_gris(
            &bases::quantifier(&es::charger_gris(entree)?, niveaux)?,
            sortie,
        ),
        Cmd::Echantillonner {
            entree,
            sortie,
            pas,
        } => es::enregistrer_gris(
            &bases::echantillonner(&es::charger_gris(entree)?, pas)?,
            sortie,
        ),
        Cmd::Pixeliser {
            entree,
            sortie,
            pas,
        } => es::enregistrer_gris(&bases::pixeliser(&es::charger_gris(entree)?, pas)?, sortie),
        Cmd::Taille { entree, bits } => {
            let img = es::charger(entree)?;
            let (largeur, hauteur) = img.dimensions();
            let bits_total = bases::taille_en_bits(largeur, hauteur, bits);
            println!("{largeur} x {hauteur} x {bits} = {bits_total} bits");
            Ok(())
        }
        Cmd::ProfilLigne { entree, sortie, y } => {
            let img = es::charger_gris(entree)?;
            enregistrer_profil(&bases::profil_ligne(&img, y), sortie)
        }
        Cmd::ProfilColonne { entree, sortie, x } => {
            let img = es::charger_gris(entree)?;
            enregistrer_profil(&bases::profil_colonne(&img, x), sortie)
        }
        Cmd::ProfilSegment {
            entree,
            sortie,
            x0,
            y0,
            x1,
            y1,
        } => {
            let img = es::charger_gris(entree)?;
            enregistrer_profil(&bases::profil_segment(&img, (x0, y0), (x1, y1)), sortie)
        }
    }
}

fn enregistrer_profil(profil: &[u8], sortie: String) -> Resultat<()> {
    println!("{} échantillons", profil.len());
    es::enregistrer_gris(&bases::image_profil(profil, 256), sortie)
}

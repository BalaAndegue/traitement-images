use clap::{Args, Subcommand, ValueEnum};
use imagerie::es;
use imagerie::morphologie::{self, ElementStructurant};
use imagerie::Resultat;

#[derive(Clone, Copy, ValueEnum)]
pub enum Forme {
    /// Croix 3x3, connexité 4.
    Croix,
    /// Carré centré. `--taille` doit être impaire (3 = connexité 8).
    Carre,
    /// Disque. `--taille` est le rayon.
    Disque,
}

#[derive(Args)]
pub struct Op {
    entree: String,
    sortie: String,
    /// Élément structurant.
    #[arg(long, value_enum, default_value = "croix")]
    forme: Forme,
    /// Taille du carré, ou rayon du disque. Ignorée pour la croix.
    #[arg(short, long, default_value_t = 3)]
    taille: usize,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Érosion binaire.
    Erosion(Op),
    /// Dilatation binaire.
    Dilatation(Op),
    /// Ouverture binaire.
    Ouverture(Op),
    /// Fermeture binaire.
    Fermeture(Op),
    /// Contour intérieur.
    GradientInterne(Op),
    /// Contour extérieur.
    GradientExterne(Op),
    /// Dilatée moins érodée.
    Gradient(Op),
    /// Érosion en niveaux de gris (minimum).
    ErosionGris(Op),
    /// Dilatation en niveaux de gris (maximum).
    DilatationGris(Op),
    /// Ouverture en niveaux de gris.
    OuvertureGris(Op),
    /// Fermeture en niveaux de gris.
    FermetureGris(Op),
    /// Gradient morphologique en niveaux de gris.
    GradientGris(Op),
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Erosion(op) => binaire(&op, morphologie::erosion),
        Cmd::Dilatation(op) => binaire(&op, morphologie::dilatation),
        Cmd::Ouverture(op) => binaire(&op, morphologie::ouverture),
        Cmd::Fermeture(op) => binaire(&op, morphologie::fermeture),
        Cmd::GradientInterne(op) => binaire(&op, morphologie::gradient_interne),
        Cmd::GradientExterne(op) => binaire(&op, morphologie::gradient_externe),
        Cmd::Gradient(op) => binaire(&op, morphologie::gradient_morphologique),
        Cmd::ErosionGris(op) => gris(&op, morphologie::erosion_gris),
        Cmd::DilatationGris(op) => gris(&op, morphologie::dilatation_gris),
        Cmd::OuvertureGris(op) => gris(&op, morphologie::ouverture_gris),
        Cmd::FermetureGris(op) => gris(&op, morphologie::fermeture_gris),
        Cmd::GradientGris(op) => gris(&op, morphologie::gradient_morphologique_gris),
    }
}

fn element(op: &Op) -> Resultat<ElementStructurant> {
    match op.forme {
        Forme::Croix => Ok(ElementStructurant::croix()),
        Forme::Carre => ElementStructurant::carre(op.taille),
        Forme::Disque => Ok(ElementStructurant::disque(op.taille)),
    }
}

fn binaire(
    op: &Op,
    f: fn(&imagerie::ImageBinaire, &ElementStructurant) -> imagerie::ImageBinaire,
) -> Resultat<()> {
    let img = es::charger_gris(&op.entree)?.map(|&p| p >= 128);
    let es = element(op)?;
    es::enregistrer_gris(&f(&img, &es).vers_gris(), &op.sortie)
}

fn gris(
    op: &Op,
    f: fn(&imagerie::ImageGris, &ElementStructurant) -> imagerie::ImageGris,
) -> Resultat<()> {
    let img = es::charger_gris(&op.entree)?;
    let es = element(op)?;
    es::enregistrer_gris(&f(&img, &es), &op.sortie)
}

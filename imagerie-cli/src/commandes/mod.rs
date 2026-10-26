use imagerie::es::{self, ImageChargee};
use imagerie::Resultat;

pub fn info(entree: &str) -> Resultat<()> {
    let img = es::charger(entree)?;
    let (l, h) = img.dimensions();
    let genre = match img {
        ImageChargee::Gris(_) => "niveaux de gris 8 bits",
        ImageChargee::Couleur(_) => "couleur RVB 24 bits",
    };
    println!("{entree} : {l} x {h} pixels, {genre}");
    Ok(())
}

pub fn convertir(entree: &str, sortie: &str, gris: bool) -> Resultat<()> {
    let img = es::charger(entree)?;
    if gris {
        es::enregistrer_gris(&img.en_gris(), sortie)
    } else {
        es::enregistrer(&img, sortie)
    }
}

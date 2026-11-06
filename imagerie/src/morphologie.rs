//! Chapitre 7 (deuxième partie) : opérateurs morphologiques.
//!

//! Responsable : Nkolo Atangana Stacy Julie.

// À retirer quand toutes les fonctions sont écrites.
#![allow(unused_variables)]

use crate::erreur::{Erreur, Resultat};
use crate::image::{Bord, ImageBinaire, ImageGris};

/// Élément structurant : liste des décalages (dx, dy) par rapport à son origine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElementStructurant {
    pub decalages: Vec<(isize, isize)>,
}

impl ElementStructurant {
    /// Croix 3x3 (connexité 4), origine comprise.
    pub fn croix() -> Self {
        let decalages = vec![(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)];
        Self { decalages }
    }

    /// Carré `taille` x `taille` centré sur l'origine.
    ///
    /// `taille` doit être impaire et au moins 1. Pour 3, c'est la 8-connexité.
    pub fn carre(taille: usize) -> Resultat<Self> {
        if taille == 0 || taille.is_multiple_of(2) {
            return Err(Erreur::Parametre(
                "La taille doit être supérieure à 0".to_string(),
            ));
        }
        let mut decalages = Vec::new();
        let centre = (taille as isize - 1) / 2;
        for y in 0..taille {
            for x in 0..taille {
                decalages.push((x as isize - centre, y as isize - centre));
            }
        }
        Ok(Self { decalages })
    }

    /// Disque de rayon `rayon` : les décalages dont dx² + dy² ≤ rayon².
    pub fn disque(rayon: usize) -> Self {
        let mut decalages = Vec::new();
        let rayon_f = rayon as f64;
        for y in -(rayon as isize)..=(rayon as isize) {
            for x in -(rayon as isize)..=(rayon as isize) {
                if (x as f64).powi(2) + (y as f64).powi(2) <= rayon_f.powi(2) {
                    decalages.push((x, y));
                }
            }
        }
        Self { decalages }
    }

    /// Élément quelconque (pas forcément symétrique) décrit par un masque binaire
    /// dont l'origine est le pixel `origine`.
    pub fn depuis_masque(masque: &ImageBinaire, origine: (usize, usize)) -> Self {
        let mut decalages = Vec::new();
        let (orig_x, orig_y) = origine;
        for y in 0..masque.hauteur() {
            for x in 0..masque.largeur() {
                if masque[(x, y)] {
                    decalages.push((x as isize - orig_x as isize, y as isize - orig_y as isize));
                }
            }
        }
        Self { decalages }
    }
}

/// Érosion : le pixel reste objet si tous les pixels sous l'élément sont objet.
pub fn erosion(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    ImageBinaire::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages.iter().all(|&(dx, dy)| {
            img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero) == Some(true)
        })
    })
}

/// Dilatation : le pixel devient objet si au moins un pixel sous l'élément est objet.
pub fn dilatation(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    ImageBinaire::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages.iter().any(|&(dx, dy)| {
            img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Zero) == Some(true)
        })
    })
}

/// Ouverture : érosion puis dilatation.
pub fn ouverture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    dilatation(&erosion(img, es), es)
}

/// Fermeture : dilatation puis érosion.
pub fn fermeture(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    erosion(&dilatation(img, es), es)
}

/// Gradient interne : image - érodée (contour intérieur).
pub fn gradient_interne(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let erodee = erosion(img, es);
    img.zip_map(&erodee, |&a, &b| a && !b)
        .expect("l'érosion garde les dimensions de l'image")
}

/// Gradient externe : dilatée - image (contour extérieur).
pub fn gradient_externe(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let dilatee = dilatation(img, es);
    img.zip_map(&dilatee, |&a, &b| !a && b)
        .expect("la dilatation garde les dimensions de l'image")
}

/// Gradient morphologique : dilatée - érodée.
pub fn gradient_morphologique(img: &ImageBinaire, es: &ElementStructurant) -> ImageBinaire {
    let dilatee = dilatation(img, es);
    let erodee = erosion(img, es);
    dilatee
        .zip_map(&erodee, |&a, &b| a && !b)
        .expect("la dilatation et l'érosion gardent les dimensions de l'image")
}

/// Érosion en niveaux de gris : minimum sous l'élément.
pub fn erosion_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages
            .iter()
            .filter_map(|&(dx, dy)| {
                img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Partiel)
            })
            .min()
            .unwrap_or(0)
    })
}

/// Dilatation en niveaux de gris : maximum sous l'élément.
pub fn dilatation_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        es.decalages
            .iter()
            .filter_map(|&(dx, dy)| {
                img.pixel_etendu(x as isize + dx, y as isize + dy, Bord::Partiel)
            })
            .max()
            .unwrap_or(0)
    })
}

pub fn ouverture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    dilatation_gris(&erosion_gris(img, es), es)
}

pub fn fermeture_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    erosion_gris(&dilatation_gris(img, es), es)
}

/// Gradient morphologique en niveaux de gris : dilatée - érodée.
pub fn gradient_morphologique_gris(img: &ImageGris, es: &ElementStructurant) -> ImageGris {
    let dilatee = dilatation_gris(img, es);
    let erodee = erosion_gris(img, es);
    ImageGris::depuis_fonction(img.largeur(), img.hauteur(), |x, y| {
        let d = dilatee.get(x, y).unwrap_or(&0);
        let e = erodee.get(x, y).unwrap_or(&0);
        d.saturating_sub(*e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erreur::Erreur;

    /// Image binaire à partir de lignes de texte ('#' = objet, '.' = fond).
    fn img(lignes: &[&str]) -> ImageBinaire {
        let h = lignes.len();
        let l = lignes[0].len();
        let donnees = lignes
            .iter()
            .flat_map(|s| s.bytes().map(|c| c == b'#'))
            .collect();
        ImageBinaire::depuis_vec(l, h, donnees).unwrap()
    }

    /// Image binaire en lignes de texte, pour comparer lisiblement.
    fn texte(i: &ImageBinaire) -> Vec<String> {
        (0..i.hauteur())
            .map(|y| {
                (0..i.largeur())
                    .map(|x| if i[(x, y)] { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    fn trie(es: &ElementStructurant) -> Vec<(isize, isize)> {
        let mut v = es.decalages.clone();
        v.sort();
        v
    }

    /// `a` est inclus dans `b` (tout pixel objet de `a` l'est dans `b`).
    fn inclus(a: &ImageBinaire, b: &ImageBinaire) -> bool {
        a.zip_map(b, |&p, &q| !p || q)
            .unwrap()
            .donnees()
            .iter()
            .all(|&v| v)
    }

    /// Image binaire d'allure irrégulière, sans bruit aléatoire.
    fn irreguliere() -> ImageBinaire {
        ImageBinaire::depuis_fonction(12, 9, |x, y| (x * 7 + y * 13 + x * y) % 5 < 2)
    }

    fn gris_irreguliere() -> ImageGris {
        ImageGris::depuis_fonction(10, 8, |x, y| ((x * 37 + y * 91 + x * y * 5) % 256) as u8)
    }

    // ----- Éléments structurants -----

    #[test]
    fn croix_a_cinq_decalages_origine_comprise() {
        let es = ElementStructurant::croix();
        assert_eq!(es.decalages.len(), 5);
        assert!(es.decalages.contains(&(0, 0)));
    }

    #[test]
    fn carre_valide() {
        let es = ElementStructurant::carre(3).unwrap();
        assert_eq!(es.decalages.len(), 9);
        assert!(es.decalages.contains(&(-1, -1)));
        assert!(es.decalages.contains(&(1, 1)));
        assert!(es.decalages.contains(&(0, 0)));
        assert_eq!(
            ElementStructurant::carre(1).unwrap().decalages,
            vec![(0, 0)]
        );
        assert_eq!(ElementStructurant::carre(5).unwrap().decalages.len(), 25);
    }

    #[test]
    fn carre_refuse_taille_nulle_ou_paire() {
        for taille in [0, 2, 4] {
            assert!(matches!(
                ElementStructurant::carre(taille),
                Err(Erreur::Parametre(_))
            ));
        }
    }

    #[test]
    fn disque_rayons_courts() {
        assert_eq!(ElementStructurant::disque(0).decalages, vec![(0, 0)]);
        // Le disque de rayon 1 est la croix.
        assert_eq!(
            trie(&ElementStructurant::disque(1)),
            trie(&ElementStructurant::croix())
        );
        // Rayon 2 : 13 points entiers dans le disque (pas les coins du carré 5x5).
        let d2 = ElementStructurant::disque(2);
        assert_eq!(d2.decalages.len(), 13);
        assert!(!d2.decalages.contains(&(2, 2)));
        assert!(d2.decalages.contains(&(2, 0)));
    }

    #[test]
    fn depuis_masque_retrouve_la_croix() {
        let masque = img(&[".#.", "###", ".#."]);
        let es = ElementStructurant::depuis_masque(&masque, (1, 1));
        assert_eq!(trie(&es), trie(&ElementStructurant::croix()));
    }

    #[test]
    fn depuis_masque_ignore_le_fond_et_decale_l_origine() {
        let masque = img(&["##.", ".#."]);
        let es = ElementStructurant::depuis_masque(&masque, (1, 0));
        assert_eq!(trie(&es), vec![(-1, 0), (0, 0), (0, 1)]);
    }

    // ----- Érosion et dilatation binaires -----

    #[test]
    fn erosion_d_un_carre_par_la_croix() {
        let i = img(&[".....", ".###.", ".###.", ".###.", "....."]);
        let r = erosion(&i, &ElementStructurant::croix());
        assert_eq!(texte(&r), [".....", ".....", "..#..", ".....", "....."]);
    }

    #[test]
    fn dilatation_d_un_pixel_isole() {
        let i = img(&[".....", ".....", "..#..", ".....", "....."]);
        let r = dilatation(&i, &ElementStructurant::croix());
        assert_eq!(texte(&r), [".....", "..#..", ".###.", "..#..", "....."]);
        let r = dilatation(&i, &ElementStructurant::carre(3).unwrap());
        assert_eq!(texte(&r), [".....", ".###.", ".###.", ".###.", "....."]);
    }

    #[test]
    fn erosion_mange_le_bord_de_l_image() {
        // Hors image = fond : un objet collé au bord est érodé de ce côté aussi.
        let i = img(&["###", "###", "###"]);
        let r = erosion(&i, &ElementStructurant::croix());
        assert_eq!(texte(&r), ["...", ".#.", "..."]);
    }

    #[test]
    fn element_reduit_a_l_origine_ne_change_rien() {
        let es = ElementStructurant {
            decalages: vec![(0, 0)],
        };
        let i = irreguliere();
        assert_eq!(erosion(&i, &es), i);
        assert_eq!(dilatation(&i, &es), i);
    }

    #[test]
    fn erosion_incluse_dans_image_incluse_dans_dilatation() {
        let i = irreguliere();
        for es in [
            ElementStructurant::croix(),
            ElementStructurant::carre(3).unwrap(),
            ElementStructurant::disque(2),
        ] {
            assert!(inclus(&erosion(&i, &es), &i));
            assert!(inclus(&i, &dilatation(&i, &es)));
        }
    }

    #[test]
    fn image_vide_et_image_pleine() {
        let vide = img(&["...", "...", "..."]);
        let es = ElementStructurant::croix();
        assert_eq!(erosion(&vide, &es), vide);
        assert_eq!(dilatation(&vide, &es), vide);
        let pleine = img(&["###", "###", "###"]);
        assert_eq!(dilatation(&pleine, &es), pleine);
    }

    // ----- Ouverture et fermeture -----

    #[test]
    fn ouverture_supprime_le_bruit_et_garde_les_gros_objets() {
        let i = img(&["###.....", "###..#..", "###....."]);
        let es = ElementStructurant::carre(3).unwrap();
        let r = ouverture(&i, &es);
        assert_eq!(texte(&r), ["###.....", "###.....", "###....."]);
    }

    #[test]
    fn ouverture_est_anti_extensive_et_idempotente() {
        let i = irreguliere();
        for es in [
            ElementStructurant::croix(),
            ElementStructurant::carre(3).unwrap(),
        ] {
            let o = ouverture(&i, &es);
            assert!(inclus(&o, &i));
            assert_eq!(ouverture(&o, &es), o);
        }
    }

    #[test]
    fn fermeture_bouche_un_trou() {
        let i = img(&[
            ".......", ".#####.", ".#####.", ".##.##.", ".#####.", ".#####.", ".......",
        ]);
        let r = fermeture(&i, &ElementStructurant::carre(3).unwrap());
        assert_eq!(
            texte(&r),
            [".......", ".#####.", ".#####.", ".#####.", ".#####.", ".#####.", "......."]
        );
    }

    // ----- Gradients -----

    fn carre_au_centre() -> ImageBinaire {
        img(&[
            ".......", ".......", "..###..", "..###..", "..###..", ".......", ".......",
        ])
    }

    #[test]
    fn gradient_interne_est_un_anneau() {
        let r = gradient_interne(&carre_au_centre(), &ElementStructurant::croix());
        assert_eq!(r.nb_pixels_objet(), 8);
        assert!(!r[(3, 3)]);
        assert!(r[(2, 2)]);
    }

    #[test]
    fn gradient_externe_est_hors_de_l_objet() {
        let i = carre_au_centre();
        let r = gradient_externe(&i, &ElementStructurant::croix());
        assert_eq!(r.nb_pixels_objet(), 12);
        // Aucun pixel du gradient externe n'appartient à l'objet.
        assert!(r
            .zip_map(&i, |&g, &p| g && p)
            .unwrap()
            .donnees()
            .iter()
            .all(|&v| !v));
        assert!(r[(1, 3)]);
    }

    #[test]
    fn gradient_morphologique_est_la_somme_des_deux() {
        let i = carre_au_centre();
        let es = ElementStructurant::croix();
        let g = gradient_morphologique(&i, &es);
        // dilatée (21 pixels) moins érodée (1 pixel).
        assert_eq!(g.nb_pixels_objet(), 20);
        let somme = gradient_interne(&i, &es)
            .zip_map(&gradient_externe(&i, &es), |&a, &b| a || b)
            .unwrap();
        assert_eq!(g, somme);
    }

    // ----- Niveaux de gris -----

    #[test]
    fn gris_uniforme_inchange_et_bords_ignores() {
        let i = ImageGris::nouvelle(3, 3, 100);
        let es = ElementStructurant::croix();
        // Les pixels hors image sont ignorés : pas de bord noir.
        assert_eq!(erosion_gris(&i, &es), i);
        assert_eq!(dilatation_gris(&i, &es), i);
        assert_eq!(
            gradient_morphologique_gris(&i, &es),
            ImageGris::nouvelle(3, 3, 0)
        );
    }

    #[test]
    fn gris_minimum_et_maximum_sous_l_element() {
        let i = ImageGris::depuis_vec(3, 3, vec![10, 10, 10, 10, 200, 10, 10, 10, 10]).unwrap();
        let es = ElementStructurant::croix();
        let d = dilatation_gris(&i, &es);
        assert_eq!(d.donnees(), &[10, 200, 10, 200, 200, 200, 10, 200, 10]);
        let e = erosion_gris(&i, &es);
        assert_eq!(e.donnees(), &[10; 9]);
        let g = gradient_morphologique_gris(&i, &es);
        assert_eq!(g.donnees(), &[0, 190, 0, 190, 190, 190, 0, 190, 0]);
    }

    #[test]
    fn gris_erosion_sous_image_sous_dilatation() {
        let i = gris_irreguliere();
        let es = ElementStructurant::carre(3).unwrap();
        let e = erosion_gris(&i, &es);
        let d = dilatation_gris(&i, &es);
        let o = ouverture_gris(&i, &es);
        let f = fermeture_gris(&i, &es);
        for y in 0..i.hauteur() {
            for x in 0..i.largeur() {
                assert!(e[(x, y)] <= i[(x, y)] && i[(x, y)] <= d[(x, y)]);
                assert!(o[(x, y)] <= i[(x, y)]);
                assert!(f[(x, y)] >= i[(x, y)]);
            }
        }
    }
}

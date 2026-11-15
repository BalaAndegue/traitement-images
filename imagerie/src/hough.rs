//! Chapitre 5 (deuxième partie) : transformée de Hough pour les droites.
//!
//! Responsable : Nkamla Chedjou Johan.
//!
//! On utilise la forme polaire rho = x cos(theta) + y sin(theta) pour éviter le
//! problème des droites verticales du plan (a, b).
//!
//! Conventions : x est la colonne, y la ligne (axe vers le bas), l'origine est
//! le coin haut-gauche. theta parcourt [-pi/2, pi/2) avec le pas demandé, rho
//! parcourt [-rho_max, rho_max] où rho_max est la diagonale de l'image.

use std::f64::consts::PI;

use crate::alea::Alea;
use crate::erreur::{Erreur, Resultat};
use crate::image::{Image, ImageBinaire, ImageGris, ImageRvb, Rvb};

/// Pas utilisé quand celui demandé n'est pas un réel strictement positif.
const PAS_THETA_DEFAUT: f64 = 1.0;
const PAS_RHO_DEFAUT: f64 = 1.0;
/// Pas minimal accepté, pour ne pas allouer un accumulateur démesuré.
const PAS_MIN: f64 = 0.01;
/// Voisinage d'exclusion autour d'un pic : deux pics plus proches que ça en theta
/// ET en rho sont considérés comme la même droite.
const VOISINAGE_THETA_DEGRES: f64 = 5.0;
const VOISINAGE_RHO: f64 = 5.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Droite {
    pub rho: f64,
    /// En radians, dans [-pi/2, pi/2).
    pub theta: f64,
    pub votes: u32,
}

/// Matrice d'accumulation (cellules de votes) : x = indice de theta, y = indice de rho.
///
/// `pas_theta` est en radians. La cellule (i, j) correspond à
/// theta = -pi/2 + i pas_theta et rho = (j - (hauteur - 1) / 2) pas_rho.
#[derive(Clone, Debug, PartialEq)]
pub struct Accumulateur {
    pub votes: Image<u32>,
    pub pas_theta: f64,
    pub pas_rho: f64,
    pub rho_max: f64,
}

impl Accumulateur {
    /// theta (radians) de la colonne `i`.
    pub fn theta(&self, i: usize) -> f64 {
        -PI / 2.0 + i as f64 * self.pas_theta
    }

    /// rho de la ligne `j`.
    pub fn rho(&self, j: usize) -> f64 {
        let centre = (self.votes.hauteur() as f64 - 1.0) / 2.0;
        (j as f64 - centre) * self.pas_rho
    }

    /// Votes de la cellule voisine (i + di, j + dj). theta est périodique : à gauche de
    /// theta = -90° on retrouve theta = +90° - pas, avec rho opposé (ligne miroir).
    fn voisin(&self, i: usize, j: usize, di: isize, dj: isize) -> Option<u32> {
        let (nt, nr) = (self.votes.largeur() as isize, self.votes.hauteur() as isize);
        let (mut ii, mut jj) = (i as isize + di, j as isize + dj);
        if ii < 0 || ii >= nt {
            ii = ii.rem_euclid(nt);
            jj = nr - 1 - jj;
        }
        if (0..nr).contains(&jj) {
            Some(self.votes[(ii as usize, jj as usize)])
        } else {
            None
        }
    }

    /// Les `nb` cellules qui ont le plus de votes (au moins `votes_min`), en évitant
    /// de renvoyer plusieurs pics voisins pour la même droite.
    ///
    /// On garde d'abord les maxima locaux (8 voisins, theta pris comme périodique), puis on les prend par votes
    /// décroissants en écartant ceux situés à moins de 5° et 5 pixels d'un pic déjà
    /// choisi. Sur un plateau de cellules égales, on renvoie le theta du milieu.
    /// theta = -90° et theta = +90° décrivent la même droite (avec rho
    /// opposé) : la comparaison en tient compte.
    pub fn pics(&self, nb: usize, votes_min: u32) -> Vec<Droite> {
        let votes_min = votes_min.max(1);
        let (nt, nr) = self.votes.dimensions();
        let mut candidats = Vec::new();
        for j in 0..nr {
            for i in 0..nt {
                let v = self.votes[(i, j)];
                if v < votes_min {
                    continue;
                }
                let mut maximum = true;
                let mut voisinage = 0u64;
                for dj in -1isize..=1 {
                    for di in -1isize..=1 {
                        if let Some(w) = self.voisin(i, j, di, dj) {
                            maximum &= w <= v;
                            voisinage += w as u64;
                        }
                    }
                }
                if maximum {
                    candidats.push((v, voisinage, i, j));
                }
            }
        }
        // Votes décroissants. Sur un plateau de cellules à égalité (droite vue sous
        // plusieurs angles voisins), la somme des votes 3 x 3 favorise le centre du
        // plateau ; à égalité complète, ordre de balayage (tri stable).
        candidats.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        let dt = VOISINAGE_THETA_DEGRES.to_radians();
        let mut droites: Vec<Droite> = Vec::new();
        let mut plateaux: Vec<(usize, usize, usize)> = Vec::new();
        for (v, _, i, j) in candidats {
            if droites.len() >= nb {
                break;
            }
            if plateaux
                .iter()
                .any(|&(pj, debut, fin)| pj == j && (debut..=fin).contains(&i))
            {
                continue;
            }
            // Une droite courte vote autant pour plusieurs theta voisins : on prend le
            // milieu du plateau de cellules égales sur la ligne de rho du pic.
            let (mut debut, mut fin) = (i, i);
            while debut > 0 && self.votes[(debut - 1, j)] == v {
                debut -= 1;
            }
            while fin + 1 < nt && self.votes[(fin + 1, j)] == v {
                fin += 1;
            }
            let d = Droite {
                rho: self.rho(j),
                theta: self.theta((debut + fin) / 2),
                votes: v,
            };
            let proche = droites.iter().any(|p| {
                let direct =
                    (p.theta - d.theta).abs() < dt && (p.rho - d.rho).abs() < VOISINAGE_RHO;
                let replie =
                    (PI - (p.theta - d.theta).abs()) < dt && (p.rho + d.rho).abs() < VOISINAGE_RHO;
                direct || replie
            });
            plateaux.push((j, debut, fin));
            if !proche {
                droites.push(d);
            }
        }
        droites
    }

    /// Image des votes ramenée sur [0, 255].
    pub fn affichable(&self) -> ImageGris {
        let max = self.votes.donnees().iter().copied().max().unwrap_or(0);
        if max == 0 {
            return self.votes.map(|_| 0);
        }
        self.votes
            .map(|&v| (255.0 * v as f64 / max as f64).round() as u8)
    }
}

/// Chaque pixel de contour vote pour toutes les droites (rho, theta) qui passent par lui.
///
/// Un pas non fini ou <= 0 est remplacé par 1 (1° ou 1 pixel) et un pas trop petit
/// est ramené à 0,01 : la signature ne permet pas de renvoyer une erreur. La ligne
/// de commande vérifie les pas avant l'appel.
pub fn hough_droites(contours: &ImageBinaire, pas_theta_degres: f64, pas_rho: f64) -> Accumulateur {
    let pas_valide = |p: f64, defaut: f64| {
        if p.is_finite() && p > 0.0 {
            p.max(PAS_MIN)
        } else {
            defaut
        }
    };
    let pas_theta_degres = pas_valide(pas_theta_degres, PAS_THETA_DEFAUT);
    let pas_rho = pas_valide(pas_rho, PAS_RHO_DEFAUT);
    let pas_theta = pas_theta_degres.to_radians();

    let (l, h) = contours.dimensions();
    let rho_max =
        ((l.saturating_sub(1) as f64).powi(2) + (h.saturating_sub(1) as f64).powi(2)).sqrt();
    // theta_i = -90° + i pas < 90°.
    let nb_theta = (180.0 / pas_theta_degres - 1e-9).ceil().max(1.0) as usize;
    let demi = (rho_max / pas_rho).ceil() as usize;
    let nb_rho = 2 * demi + 1;

    let (cos, sin): (Vec<f64>, Vec<f64>) = (0..nb_theta)
        .map(|i| {
            let t = -PI / 2.0 + i as f64 * pas_theta;
            (t.cos(), t.sin())
        })
        .unzip();
    let mut votes = Image::nouvelle(nb_theta, nb_rho, 0u32);
    for (x, y, &objet) in contours.pixels() {
        if !objet {
            continue;
        }
        let (xf, yf) = (x as f64, y as f64);
        for i in 0..nb_theta {
            let rho = xf * cos[i] + yf * sin[i];
            let j = (rho / pas_rho).round() as isize + demi as isize;
            if (0..nb_rho as isize).contains(&j) {
                votes[(i, j as usize)] += 1;
            }
        }
    }
    Accumulateur {
        votes,
        pas_theta,
        pas_rho,
        rho_max,
    }
}

/// Trace les droites détectées en couleur par-dessus l'image.
///
/// Pour une droite plutôt horizontale (|sin theta| >= |cos theta|) on calcule y
/// pour chaque colonne x, sinon x pour chaque ligne y : le tracé n'a pas de trou.
pub fn tracer_droites(img: &ImageGris, droites: &[Droite], couleur: Rvb) -> ImageRvb {
    let mut sortie = img.map(|&g| Rvb::new(g, g, g));
    let (l, h) = img.dimensions();
    for d in droites {
        let (c, s) = (d.theta.cos(), d.theta.sin());
        if s.abs() >= c.abs() {
            for x in 0..l {
                let y = ((d.rho - x as f64 * c) / s).round();
                if y >= 0.0 && (y as usize) < h {
                    sortie[(x, y as usize)] = couleur;
                }
            }
        } else {
            for y in 0..h {
                let x = ((d.rho - y as f64 * s) / c).round();
                if x >= 0.0 && (x as usize) < l {
                    sortie[(x as usize, y)] = couleur;
                }
            }
        }
    }
    sortie
}

/// Image de test de l'exemple du cours : `nb_alignes` points régulièrement espacés sur
/// la droite qui descend du coin (10 %, 10 %) au coin (90 %, 90 %) de l'image, plus
/// `nb_aleatoires` points tirés au hasard (graine fixe : figure reproductible).
pub fn image_points(
    largeur: usize,
    hauteur: usize,
    nb_alignes: usize,
    nb_aleatoires: usize,
    graine: u64,
) -> Resultat<ImageBinaire> {
    if largeur < 2 || hauteur < 2 {
        return Err(Erreur::Parametre(format!(
            "image {largeur}x{hauteur} trop petite pour y placer des points"
        )));
    }
    if nb_alignes + nb_aleatoires > largeur * hauteur {
        return Err(Erreur::Parametre(
            "plus de points demandés que de pixels".into(),
        ));
    }
    let mut img = ImageBinaire::nouvelle(largeur, hauteur, false);
    let (x0, y0) = (0.1 * (largeur - 1) as f64, 0.1 * (hauteur - 1) as f64);
    let (x1, y1) = (0.9 * (largeur - 1) as f64, 0.9 * (hauteur - 1) as f64);
    for k in 0..nb_alignes {
        let t = if nb_alignes == 1 {
            0.5
        } else {
            k as f64 / (nb_alignes - 1) as f64
        };
        let x = (x0 + t * (x1 - x0)).round() as usize;
        let y = (y0 + t * (y1 - y0)).round() as usize;
        img[(x, y)] = true;
    }
    let mut alea = Alea::new(graine);
    let mut places = 0;
    while places < nb_aleatoires {
        let (x, y) = (alea.entier(largeur), alea.entier(hauteur));
        if !img[(x, y)] {
            img[(x, y)] = true;
            places += 1;
        }
    }
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(l: usize, h: usize, liste: &[(usize, usize)]) -> ImageBinaire {
        let mut img = ImageBinaire::nouvelle(l, h, false);
        for &p in liste {
            img[p] = true;
        }
        img
    }

    #[test]
    fn dimensions_accumulateur() {
        let acc = hough_droites(&ImageBinaire::nouvelle(4, 5, false), 1.0, 1.0);
        // rho_max = sqrt(3² + 4²) = 5 -> rho de -5 à 5 : 11 lignes ; 180 colonnes.
        assert_eq!(acc.rho_max, 5.0);
        assert_eq!(acc.votes.dimensions(), (180, 11));
        assert_eq!(acc.theta(0), -PI / 2.0);
        assert_eq!(acc.rho(0), -5.0);
        assert_eq!(acc.rho(5), 0.0);
        assert!(acc.votes.donnees().iter().all(|&v| v == 0));
        // Pas invalide : on retombe sur 1° et 1 pixel au lieu de paniquer.
        let acc = hough_droites(&ImageBinaire::nouvelle(4, 5, false), 0.0, f64::NAN);
        assert_eq!(acc.votes.dimensions(), (180, 11));
    }

    #[test]
    fn un_point_vote_une_fois_par_theta() {
        let acc = hough_droites(&points(10, 10, &[(3, 4)]), 1.0, 1.0);
        let total: u32 = acc.votes.donnees().iter().sum();
        assert_eq!(total, 180);
        // theta = 0 : rho = x = 3 ; theta = -90° : rho = -y = -4.
        let i0 = 90;
        let j = (0..acc.votes.hauteur())
            .find(|&j| acc.votes[(i0, j)] == 1)
            .unwrap();
        assert!((acc.rho(j) - 3.0).abs() < 1e-9);
        let j = (0..acc.votes.hauteur())
            .find(|&j| acc.votes[(0, j)] == 1)
            .unwrap();
        assert!((acc.rho(j) + 4.0).abs() < 1e-9);
    }

    #[test]
    fn droite_verticale() {
        // Colonne x = 6 : theta = 0, rho = 6, 10 votes.
        let liste: Vec<_> = (0..10).map(|y| (6, y)).collect();
        let acc = hough_droites(&points(12, 10, &liste), 1.0, 1.0);
        let pics = acc.pics(1, 5);
        assert_eq!(pics.len(), 1);
        assert_eq!(pics[0].votes, 10);
        assert!(pics[0].theta.abs() < 1e-9);
        assert!((pics[0].rho - 6.0).abs() < 1e-9);
    }

    #[test]
    fn droite_horizontale_et_diagonale() {
        // Ligne y = 2 (theta = -90°, rho = -2) et diagonale y = x (theta = -45°, rho = 0).
        let mut liste: Vec<_> = (0..16).map(|x| (x, 2)).collect();
        liste.extend((0..16).map(|k| (k, k)));
        let acc = hough_droites(&points(16, 16, &liste), 1.0, 1.0);
        let pics = acc.pics(2, 8);
        assert_eq!(pics.len(), 2);
        let horiz = pics
            .iter()
            .find(|d| (d.theta + PI / 2.0).abs() < 1e-9)
            .expect("droite horizontale");
        assert!((horiz.rho + 2.0).abs() < 1e-9);
        assert!(horiz.votes >= 16);
        let diag = pics
            .iter()
            .find(|d| (d.theta + PI / 4.0).abs() < 1e-9)
            .expect("diagonale");
        assert!(diag.rho.abs() < 1e-9);
        assert_eq!(diag.votes, 16);
    }

    #[test]
    fn pics_sans_doublon_et_votes_min() {
        let liste: Vec<_> = (0..20).map(|y| (5, y)).collect();
        let acc = hough_droites(&points(20, 20, &liste), 1.0, 1.0);
        // Une seule droite : même en demandant 5 pics avec un seuil haut, un seul sort.
        let pics = acc.pics(5, 15);
        assert_eq!(pics.len(), 1);
        // Seuil plus haut que le maximum : aucun pic.
        assert!(acc.pics(5, 21).is_empty());
        assert!(acc.pics(0, 1).is_empty());
    }

    #[test]
    fn accumulateur_affichable() {
        let mut votes = Image::nouvelle(3, 1, 0u32);
        votes[(1, 0)] = 2;
        votes[(2, 0)] = 4;
        let acc = Accumulateur {
            votes,
            pas_theta: 1.0,
            pas_rho: 1.0,
            rho_max: 0.0,
        };
        assert_eq!(acc.affichable().donnees(), &[0, 128, 255]);
        let vide = hough_droites(&ImageBinaire::nouvelle(3, 3, false), 1.0, 1.0);
        assert!(vide.affichable().donnees().iter().all(|&v| v == 0));
    }

    #[test]
    fn trace_horizontal_et_vertical() {
        let img = ImageGris::nouvelle(5, 4, 100);
        let rouge = Rvb::new(255, 0, 0);
        let droites = [
            Droite {
                rho: 2.0,
                theta: 0.0,
                votes: 1,
            },
            Droite {
                rho: -1.0,
                theta: -PI / 2.0,
                votes: 1,
            },
        ];
        let t = tracer_droites(&img, &droites, rouge);
        for y in 0..4 {
            assert_eq!(t[(2, y)], rouge);
        }
        for x in 0..5 {
            assert_eq!(t[(x, 1)], rouge);
        }
        assert_eq!(t[(0, 0)], Rvb::new(100, 100, 100));
        assert_eq!(t[(4, 3)], Rvb::new(100, 100, 100));
    }

    #[test]
    fn points_alignes_contre_aleatoires() {
        // Exemple du cours : les points alignés donnent un pic net, pas les aléatoires.
        let alignes = image_points(100, 100, 20, 0, 1).unwrap();
        assert_eq!(alignes.nb_pixels_objet(), 20);
        let acc = hough_droites(&alignes, 1.0, 1.0);
        let pic = acc.pics(1, 1)[0];
        assert!(pic.votes >= 18, "{pic:?}");
        // Droite y = x : theta = -45°, rho = 0.
        assert!((pic.theta.to_degrees() + 45.0).abs() <= 1.0);
        assert!(pic.rho.abs() <= 1.0);

        let aleatoires = image_points(100, 100, 0, 20, 1).unwrap();
        assert_eq!(aleatoires.nb_pixels_objet(), 20);
        let max = *hough_droites(&aleatoires, 1.0, 1.0)
            .votes
            .donnees()
            .iter()
            .max()
            .unwrap();
        assert!(max <= 5, "pic de {max} votes pour des points aléatoires");

        assert!(image_points(1, 10, 1, 0, 1).is_err());
        assert!(image_points(2, 2, 3, 2, 1).is_err());
    }
}

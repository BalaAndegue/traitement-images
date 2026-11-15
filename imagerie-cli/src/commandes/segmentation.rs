use std::collections::HashSet;

use clap::{Args, Subcommand, ValueEnum};
use imagerie::es;
use imagerie::segmentation::{self, Homogeneite};
use imagerie::{Connexite, Erreur, ImageGris, ImageRvb, Resultat, Rvb};

/// Entrée et sortie d'un seuillage automatique.
#[derive(Args)]
pub struct EntreeSortie {
    entree: String,
    /// Image binaire résultat (objet en blanc).
    sortie: String,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum Critere {
    /// Variance du bloc <= valeur.
    Variance,
    /// max - min du bloc <= valeur.
    MaxMin,
}

#[derive(Args)]
pub struct OptionsCritere {
    /// Prédicat d'homogénéité.
    #[arg(long, value_enum, default_value = "variance")]
    critere: Critere,
    /// Seuil du prédicat (variance, ou écart max - min entre 0 et 255).
    #[arg(long, default_value_t = 100.0)]
    valeur: f64,
    /// Côté minimal des blocs du quadtree.
    #[arg(long, default_value_t = 2)]
    taille_min: usize,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Seuillage simple : blanc si f(x, y) >= seuil.
    Seuil {
        entree: String,
        sortie: String,
        seuil: u8,
    },
    /// Multi-seuils : n seuils croissants donnent n + 1 classes (niveaux de gris répartis).
    Multiple {
        entree: String,
        sortie: String,
        #[arg(required = true)]
        seuils: Vec<u8>,
    },
    /// Seuil = moyenne des niveaux de gris.
    Moyenne(EntreeSortie),
    /// Seuil = milieu de la dynamique (min + max) / 2.
    Milieu(EntreeSortie),
    /// Seuil = médiane (équilibre les deux parties de l'histogramme).
    Mediane(EntreeSortie),
    /// Seuillage automatique par la méthode d'Otsu.
    Otsu(EntreeSortie),
    /// Seuillage automatique par la méthode de Kittler (erreur minimale).
    Kittler(EntreeSortie),
    /// Seuillage local adaptatif par blocs (Otsu par bloc).
    Adaptatif {
        entree: String,
        sortie: String,
        /// Côté des blocs en pixels.
        #[arg(short, long, default_value_t = 64)]
        bloc: usize,
        /// Variance en dessous de laquelle un bloc n'est pas seuillé.
        #[arg(long, default_value_t = 100.0)]
        variance_min: f64,
    },
    /// k-moyennes sur l'intensité : chaque pixel prend le niveau de son centre.
    Kmoyennes {
        entree: String,
        sortie: String,
        #[arg(short, default_value_t = 3)]
        k: usize,
        #[arg(long, default_value_t = 100)]
        iterations: usize,
        #[arg(long, default_value_t = 1)]
        graine: u64,
    },
    /// k-moyennes sur la couleur (RVB) : image « posterisée ».
    KmoyennesCouleur {
        entree: String,
        sortie: String,
        #[arg(short, default_value_t = 4)]
        k: usize,
        #[arg(long, default_value_t = 100)]
        iterations: usize,
        #[arg(long, default_value_t = 1)]
        graine: u64,
    },
    /// Phase de division seule : dessine les blocs du quadtree en rouge.
    Division {
        entree: String,
        sortie: String,
        #[command(flatten)]
        critere: OptionsCritere,
    },
    /// Division-fusion : une couleur par région.
    DivisionFusion {
        entree: String,
        sortie: String,
        #[command(flatten)]
        critere: OptionsCritere,
    },
    /// Croissance de régions : une couleur par région.
    Croissance {
        entree: String,
        sortie: String,
        /// Germe x,y (option répétable). Sans germe, germes automatiques.
        #[arg(short, long, value_parser = lire_germe)]
        germe: Vec<(usize, usize)>,
        /// Écart maximal entre un pixel et la moyenne de la région.
        #[arg(short, long, default_value_t = 20.0)]
        tolerance: f64,
        /// Espacement des germes automatiques.
        #[arg(long, default_value_t = 32)]
        espacement: usize,
        /// Connexité 8 (4 par défaut).
        #[arg(long)]
        huit: bool,
    },
    /// Germes automatiques (minima du gradient), marqués d'une croix rouge.
    Germes {
        entree: String,
        sortie: String,
        #[arg(long, default_value_t = 32)]
        espacement: usize,
    },
}

fn lire_germe(s: &str) -> Result<(usize, usize), String> {
    let (x, y) = s
        .split_once(',')
        .ok_or_else(|| format!("germe « {s} » : attendu x,y"))?;
    let x = x.trim().parse().map_err(|_| format!("x invalide : {x}"))?;
    let y = y.trim().parse().map_err(|_| format!("y invalide : {y}"))?;
    Ok((x, y))
}

fn homogeneite(o: &OptionsCritere) -> Resultat<Homogeneite> {
    match o.critere {
        Critere::Variance => Ok(Homogeneite::Variance(o.valeur)),
        Critere::MaxMin => {
            if o.valeur.fract() != 0.0 || !(0.0..=255.0).contains(&o.valeur) {
                return Err(Erreur::Parametre(format!(
                    "écart max - min invalide : {} (entier entre 0 et 255)",
                    o.valeur
                )));
            }
            Ok(Homogeneite::MaxMin(o.valeur as u8))
        }
    }
}

fn seuil_auto(op: &EntreeSortie, nom: &str, f: fn(&ImageGris) -> u8) -> Resultat<()> {
    let img = es::charger_gris(&op.entree)?;
    let t = f(&img);
    println!("seuil ({nom}) : {t}");
    es::enregistrer_gris(&segmentation::seuillage(&img, t).vers_gris(), &op.sortie)
}

fn en_couleur(img: &ImageGris) -> ImageRvb {
    img.map(|&g| Rvb::new(g, g, g))
}

pub fn executer(cmd: Cmd) -> Resultat<()> {
    match cmd {
        Cmd::Seuil {
            entree,
            sortie,
            seuil,
        } => {
            let img = es::charger_gris(entree)?;
            es::enregistrer_gris(&segmentation::seuillage(&img, seuil).vers_gris(), sortie)
        }
        Cmd::Multiple {
            entree,
            sortie,
            seuils,
        } => {
            let img = es::charger_gris(entree)?;
            let classes = segmentation::seuillage_multiple(&img, &seuils)?;
            es::enregistrer_gris(&classes.vers_gris(), sortie)
        }
        Cmd::Moyenne(op) => seuil_auto(&op, "moyenne", segmentation::seuil_moyenne),
        Cmd::Milieu(op) => seuil_auto(&op, "milieu", segmentation::seuil_milieu),
        Cmd::Mediane(op) => seuil_auto(&op, "médiane", segmentation::seuil_median),
        Cmd::Otsu(op) => seuil_auto(&op, "Otsu", segmentation::seuil_otsu),
        Cmd::Kittler(op) => seuil_auto(&op, "Kittler", segmentation::seuil_kittler),
        Cmd::Adaptatif {
            entree,
            sortie,
            bloc,
            variance_min,
        } => {
            let img = es::charger_gris(entree)?;
            let b = segmentation::seuillage_adaptatif(&img, bloc, variance_min)?;
            es::enregistrer_gris(&b.vers_gris(), sortie)
        }
        Cmd::Kmoyennes {
            entree,
            sortie,
            k,
            iterations,
            graine,
        } => {
            let img = es::charger_gris(entree)?;
            let r = segmentation::kmoyennes_gris(&img, k, iterations, graine)?;
            let centres: Vec<String> = r.centres.iter().map(|c| format!("{c:.1}")).collect();
            println!(
                "{} itérations, centres : {}",
                r.iterations,
                centres.join(", ")
            );
            let niveaux = r
                .etiquettes
                .map(|&e| r.centres[e as usize - 1].round().clamp(0.0, 255.0) as u8);
            es::enregistrer_gris(&niveaux, sortie)
        }
        Cmd::KmoyennesCouleur {
            entree,
            sortie,
            k,
            iterations,
            graine,
        } => {
            let img = es::charger_couleur(entree)?;
            let r = segmentation::kmoyennes_couleur(&img, k, iterations, graine)?;
            let centres: Vec<String> = r
                .centres
                .iter()
                .map(|c| format!("({:.0}, {:.0}, {:.0})", c[0], c[1], c[2]))
                .collect();
            println!(
                "{} itérations, centres : {}",
                r.iterations,
                centres.join(", ")
            );
            es::enregistrer_couleur(&segmentation::image_kmoyennes_couleur(&r), sortie)
        }
        Cmd::Division {
            entree,
            sortie,
            critere,
        } => {
            let img = es::charger_gris(entree)?;
            let h = homogeneite(&critere)?;
            if critere.taille_min == 0 {
                return Err(Erreur::Parametre(
                    "--taille-min doit être au moins 1".into(),
                ));
            }
            let blocs = segmentation::division(&img, h, critere.taille_min);
            println!("{} blocs", blocs.len());
            let mut trace = en_couleur(&img);
            let rouge = Rvb::new(255, 0, 0);
            for (x0, y0, l, hb) in blocs {
                for x in x0..x0 + l {
                    trace[(x, y0)] = rouge;
                }
                for y in y0..y0 + hb {
                    trace[(x0, y)] = rouge;
                }
            }
            es::enregistrer_couleur(&trace, sortie)
        }
        Cmd::DivisionFusion {
            entree,
            sortie,
            critere,
        } => {
            let img = es::charger_gris(entree)?;
            let h = homogeneite(&critere)?;
            let e = segmentation::division_fusion(&img, h, critere.taille_min)?;
            println!("{} régions", e.etiquette_max());
            es::enregistrer_couleur(&e.vers_couleurs(), sortie)
        }
        Cmd::Croissance {
            entree,
            sortie,
            germe,
            tolerance,
            espacement,
            huit,
        } => {
            let img = es::charger_gris(entree)?;
            let germes = if germe.is_empty() {
                let g = segmentation::germes_automatiques(&img, espacement);
                println!("{} germes automatiques", g.len());
                g
            } else {
                germe
            };
            let connexite = if huit {
                Connexite::Huit
            } else {
                Connexite::Quatre
            };
            let e = segmentation::croissance_regions(&img, &germes, tolerance, connexite)?;
            let non_classes = e.donnees().iter().filter(|&&v| v == 0).count();
            let regions: HashSet<u32> = e.donnees().iter().copied().filter(|&v| v > 0).collect();
            println!(
                "{} régions, {non_classes} pixels non classés",
                regions.len()
            );
            es::enregistrer_couleur(&e.vers_couleurs(), sortie)
        }
        Cmd::Germes {
            entree,
            sortie,
            espacement,
        } => {
            let img = es::charger_gris(entree)?;
            let germes = segmentation::germes_automatiques(&img, espacement);
            println!("{} germes : {germes:?}", germes.len());
            let mut trace = en_couleur(&img);
            let rouge = Rvb::new(255, 0, 0);
            for &(x, y) in &germes {
                for d in -2isize..=2 {
                    for (px, py) in [(x as isize + d, y as isize), (x as isize, y as isize + d)] {
                        if trace.contient(px, py) {
                            trace[(px as usize, py as usize)] = rouge;
                        }
                    }
                }
            }
            es::enregistrer_couleur(&trace, sortie)
        }
    }
}

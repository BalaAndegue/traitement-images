use std::fmt;

pub type Resultat<T> = Result<T, Erreur>;

#[derive(Debug)]
pub enum Erreur {
    /// Problème de lecture ou d'écriture de fichier.
    Io(std::io::Error),
    /// Fichier d'image mal formé ou format non supporté.
    Format(String),
    /// Tailles d'images incompatibles.
    Dimensions(String),
    /// Paramètre invalide (taille de noyau paire, seuil hors bornes...).
    Parametre(String),
}

impl fmt::Display for Erreur {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Erreur::Io(e) => write!(f, "erreur d'entrée/sortie : {e}"),
            Erreur::Format(m) => write!(f, "format invalide : {m}"),
            Erreur::Dimensions(m) => write!(f, "dimensions incompatibles : {m}"),
            Erreur::Parametre(m) => write!(f, "paramètre invalide : {m}"),
        }
    }
}

impl std::error::Error for Erreur {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Erreur::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Erreur {
    fn from(e: std::io::Error) -> Self {
        Erreur::Io(e)
    }
}

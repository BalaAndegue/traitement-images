//! Bibliothèque de traitement d'images réalisée pour le cours du Dr Tapamo.
//!
//! Un module par partie du cours :
//!
//! | Module           | Chapitre                              | Responsable      |
//! |------------------|---------------------------------------|------------------|
//! | `image`, `es`    | Noyau : types d'images, fichiers      | Bala             |
//! | `bases`          | 1. Introduction                       | Bala             |
//! | `ponctuel`       | 2. Histogramme, contraste             | Jeff             |
//! | `arithmetique`   | 2. Opérations logiques/arithmétiques  | Jeff             |
//! | `interpolation`  | 2. Interpolation, changement d'échelle| Pierre Marcelle  |
//! | `filtrage`       | 3. Convolution, filtres               | Pierre Marcelle  |
//! | `fourier`        | 4. Transformée de Fourier             | Pio              |
//! | `contours`       | 5. Dérivées, gradient, laplacien      | Pio              |
//! | `hough`          | 5. Transformée de Hough               | Johan            |
//! | `segmentation`   | 6. Segmentation                       | Johan            |
//! | `binaire`        | 7. Images binaires                    | Stacy            |
//! | `morphologie`    | 7. Opérateurs morphologiques          | Stacy            |

pub mod erreur;

pub use erreur::{Erreur, Resultat};

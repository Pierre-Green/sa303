//! Géométrie du bumper de capotage : silhouette rigidement fixée à l'enceinte
//! du haut en vol, et hauteur de la manille. Le rendu du bumper en stack
//! (toujours parallèle au sol, jamais fixé à l'enceinte) n'est pas ici : il
//! dépend de la grappe assemblée, pas seulement de l'enceinte, et vit dans
//! `crate::cluster::solver`.

use super::model::BumperModel;
use crate::speaker::SpeakerModel;
use crate::vector::Vec2;

/// Silhouette du bumper (rectangle), repère de l'enceinte du haut — tangent à
/// sa face supérieure. En vol le bumper est rigidement fixé à l'enceinte
/// (même système que le bras de pivot entre deux enceintes), donc tourne
/// avec elle ; en stack il reste au contraire toujours parallèle au sol —
/// c'est l'enceinte de référence qui prend l'angle, pas le bumper — et se
/// calcule donc séparément (voir `crate::cluster::solver`), sans cette fonction.
pub fn bumper_outline_top(speaker: &SpeakerModel, bumper: &BumperModel) -> [Vec2; 4] {
    let half_d = bumper.depth / 2.0;
    let speaker_half_h = speaker.mechanical.height / 2.0;
    let (y0, y1) = (speaker_half_h, speaker_half_h + bumper.height);
    [
        Vec2::new(-half_d, y1),
        Vec2::new(half_d, y1),
        Vec2::new(half_d, y0),
        Vec2::new(-half_d, y0),
    ]
}

/// Hauteur du point d'accroche (la manille) au-dessus du centre de l'enceinte
/// du haut — le bumper posé sur l'enceinte, la manille se fermant au-dessus.
pub fn bumper_pickup_height(speaker: &SpeakerModel, bumper: &BumperModel) -> f64 {
    speaker.mechanical.height / 2.0 + bumper.height + bumper.shackle_height_above_bumper
}

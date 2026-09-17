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

/// Les deux pions du bumper — avant puis arrière — dans le repère de la
/// silhouette qu'on lui passe.
///
/// `outline` est donné dans l'ordre de `bumper_outline_top` : \[avant-haut,
/// arrière-haut, arrière-bas, avant-bas\]. On interpole le long des arêtes
/// plutôt que d'ajouter des offsets sur `x` et `y` : la silhouette est déjà
/// tournée avec l'enceinte en vol, donc raisonner en « avant / arrière / bas »
/// sur ses propres arêtes reste juste quelle que soit son assiette, là où un
/// décalage en `x` supposerait le bumper à plat.
pub fn bumper_pin_points(outline: &[Vec2; 4], bumper: &BumperModel) -> [Vec2; 2] {
    let [front_top, rear_top, rear_bottom, front_bottom] = *outline;
    // Arête du bas, de l'avant vers l'arrière : elle porte la cote de recul.
    let along = rear_bottom - front_bottom;
    let depth = along.norm();
    let along_unit = if depth > 0.0 {
        along * (1.0 / depth)
    } else {
        Vec2::ZERO
    };
    // Montée vers le haut du bumper : elle porte la cote de hauteur.
    let up_front = front_top - front_bottom;
    let thickness = up_front.norm();
    let up_unit = if thickness > 0.0 {
        up_front * (1.0 / thickness)
    } else {
        Vec2::ZERO
    };
    let _ = rear_top;
    let rise = up_unit * bumper.pins.height_from_bottom_mm;
    [
        front_bottom + along_unit * bumper.pins.front_from_front_mm + rise,
        rear_bottom - along_unit * bumper.pins.rear_from_rear_mm + rise,
    ]
}

/// Hauteur du point d'accroche (la manille) au-dessus du centre de l'enceinte
/// du haut — le bumper posé sur l'enceinte, la manille se fermant au-dessus.
pub fn bumper_pickup_height(speaker: &SpeakerModel, bumper: &BumperModel) -> f64 {
    speaker.mechanical.height / 2.0 + bumper.height + bumper.shackle_height_above_bumper
}

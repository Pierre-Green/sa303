//! Bumper de capotage : posé sur l'enceinte du haut en vol, ou sous
//! l'enceinte du bas en stack. Sa compatibilité est déclarée par enceinte et
//! par compartiment (un bumper peut être prévu pour le vol sans être posable
//! au sol, ou l'inverse).

use serde::{Deserialize, Serialize};

/// Un bumper est utilisable sur une enceinte donnée, éventuellement seulement
/// dans un des deux compartiments (ex. un bumper prévu pour le vol mais pas
/// posable au sol).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperCompatibility {
    pub speaker_model_id: String,
    pub flown: bool,
    pub stacked: bool,
}

/// Position des deux pions qui goupillent le bumper à l'enceinte de référence.
///
/// Cotée **depuis les bords du bumper**, comme sur le plan : l'avant pour le
/// pion avant, l'arrière pour le pion arrière. C'est du perçage déclaré, pas
/// une construction — les déduire de la quincaillerie de l'enceinte donnerait
/// deux points qui ne sont pas ceux-là, et qui bougeraient avec le modèle
/// d'enceinte monté dessous.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperPins {
    /// Recul du pion avant depuis la face avant du bumper, mm.
    pub front_from_front_mm: f64,
    /// Avancée du pion arrière depuis la face arrière du bumper, mm.
    pub rear_from_rear_mm: f64,
    /// Hauteur des deux pions au-dessus du dessous du bumper, mm. Les deux sont
    /// sur la même ligne : c'est un perçage de flanc, traversant.
    ///
    /// **Cote de dessin, et de stack.** En vol, elle ne place plus rien : le
    /// bumper y est calé par sa bielle et par sa barre, qui sont les deux
    /// liaisons réelles. Elle est relevée au mm près là où la fermeture
    /// géométrique se joue au centième — la garder pour placer le pion avant
    /// désalignait le trou haut de la barre de 6,3 mm.
    pub height_from_bottom_mm: f64,
}

/// Barre arrière du bumper : elle descend du trou haut percé dans le bumper et
/// se boulonne au caisson de référence par sa paire ancrage/verrou, exactement
/// comme la barre arrière d'une jonction. C'est elle qui verrouille l'assiette,
/// donc c'est son bras qui fixe ce que la paire encaisse.
///
/// Elle existe en plusieurs longueurs, une par inclinaison — c'est ainsi qu'on
/// penche la première tête en stack. La barre montée en vol est celle à 0°.
///
/// Seul le trou haut est coté ici, et **depuis l'ancrage** : c'est le bras
/// utile, le seul que la statique demande. Les bouts de la barre ne changent
/// rien aux efforts ; les coter obligerait à déclarer une longueur et un calage
/// d'origine qui ne servent qu'au dessin.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperRearBar {
    /// Inclinaison de la barre **par rapport au bumper**, degrés. 0° en vol.
    pub tilt_deg: f64,
    /// Abscisse du trou haut depuis l'ancrage, le long de l'axe de la paire
    /// (ancrage → trou haut), mm.
    pub top_hole_along_mm: f64,
    /// Déport du trou haut en travers de ce même axe, mm, positif **vers
    /// l'avant** du caisson — même convention que `up660` sur la barre de
    /// jonction, qui n'est pas non plus sur l'axe.
    pub top_hole_lateral_mm: f64,
}

/// Bumper de capotage, posé sur l'enceinte du haut en vol ou sous l'enceinte
/// du bas en stack. En vol, il s'accroche par ses trous de manille, ou par la
/// barre de déport goupillée dans ses trous de liaison.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperModel {
    pub id: String,
    pub name: String,
    /// Profondeur avant-arrière du bumper, mm.
    pub depth: f64,
    /// Épaisseur du bumper, mm.
    pub height: f64,
    /// Perçage des deux pions, coté depuis les bords du bumper.
    pub pins: BumperPins,
    /// Longueur **utile** de la bielle de pivot avant, mm : entraxe entre sa
    /// goupille basse (la charnière haute du caisson, `SpeakerGeometry::ht`) et
    /// sa goupille haute, dans le bumper.
    ///
    /// C'est elle qui place le bumper en vol, avec le trou haut de la barre
    /// arrière. Les deux ensemble ferment la géométrie : l'écart entre les deux
    /// goupilles hautes doit retomber sur l'entraxe des pions du plan, et il y
    /// retombe à 2 µm sur la SA303. Placer le bumper à `height_from_bottom_mm`
    /// ne fermait pas — 6,3 mm d'écart, parce que cette cote-là est arrondie.
    /// Vérifié par `the_bumper_geometry_closes_on_the_speaker`.
    pub pivot_bar_usable_length_mm: f64,
    /// Barres arrière disponibles, une par inclinaison.
    pub rear_bars: Vec<BumperRearBar>,
    /// Trous de manille et trous de liaison de la barre de déport : les seuls
    /// points d'accroche possibles en vol.
    pub rigging: super::BumperRigging,
    pub compatible_speakers: Vec<BumperCompatibility>,
}

/// Tolérance de comparaison sur l'inclinaison déclarée d'une barre arrière :
/// les valeurs du plan sont des entiers de degré, jamais des flottants à
/// rapprocher au plus près.
const TILT_MATCH_TOLERANCE_DEG: f64 = 1e-6;

impl BumperModel {
    /// La barre arrière à l'inclinaison demandée, si elle est déclarée. Aucun
    /// rattrapage à la barre la plus proche : monter une barre de 10° là où le
    /// plan en demande une de 20° change l'assiette réelle de la tête, ce n'est
    /// pas au solveur d'en décider.
    pub fn rear_bar_at_tilt(&self, tilt_deg: f64) -> Option<&BumperRearBar> {
        self.rear_bars
            .iter()
            .find(|bar| (bar.tilt_deg - tilt_deg).abs() < TILT_MATCH_TOLERANCE_DEG)
    }
}

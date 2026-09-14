//! Rapport de géométrie d'une `SpeakerModel` (brief §3, §9) : recoupe le bras
//! de levier de chaque trou de la grille percée contre la formule
//! trigonométrique de contrôle, et échoue si l'écart dépasse la tolérance —
//! une géométrie qui ne recoupe pas est physiquement invalide, pas un simple
//! avertissement. Sert la page "Équipement et enceinte", en lecture seule
//! côté front.

use super::geometry::{
    check_rear_bar, speaker_outline, BarWarning, CrownRow, JointOffset, SpeakerGeometry,
};
use super::model::SpeakerModel;
use crate::vector::Vec2;
use serde::Serialize;

/// Tolérance de recoupement du bras de levier (brief §3) : au-delà, la géométrie
/// est incohérente et le calcul doit remonter une erreur, pas un résultat.
pub const LEVER_TOLERANCE_MM: f64 = 0.5;

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CrownHoleReport {
    pub splay_deg: f64,
    pub row: CrownRow,
    pub radius: f64,
    pub position: Vec2,
    /// Goupille basse de la bielle avant à ce splay : le centre depuis lequel
    /// ce trou est percé. Il bouge d'un trou à l'autre — les huit trous ne sont
    /// pas sur un arc centré sur un point fixe (brief §4).
    pub pv: Vec2,
    /// Écartement des coins avant à ce splay (brief §5).
    pub offset: JointOffset,
    pub lever_mm: f64,
    pub lever_check_mm: f64,
    pub discrepancy_mm: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerGeometryReport {
    pub ha: f64,
    pub ht: Vec2,
    pub hb: Vec2,
    /// Goupille basse de la bielle au splay 0. Ce n'est **pas** un pivot fixe :
    /// voir `CrownHoleReport::pv` pour sa position à chaque cran.
    pub pv0: Vec2,
    pub anchor_local: Vec2,
    pub latch_local: Vec2,
    pub front_edge: Vec2,
    pub bielle_entraxe: f64,
    /// Écarts relevés entre la barre arrière déclarée et la géométrie de
    /// jonction qu'elle dessert, plus les distances au bord en dessous du
    /// minimum. Vide = rien à signaler. Ce n'est jamais bloquant : une barre
    /// franchement incompatible remonte une `GeometryInconsistency`, pas un
    /// avertissement.
    pub bar_warnings: Vec<BarWarning>,
    /// Un par trou percé de `splay_grid`.
    pub holes: Vec<CrownHoleReport>,
    /// Silhouette de l'enceinte (trapèze), repère enceinte — pour
    /// l'`ArrayViewer` uniquement, afin qu'il n'ait aucune trigonométrie à
    /// faire lui-même.
    pub outline: [Vec2; 4],
}

/// Incohérence de recoupement sur un trou donné (brief §3) : ce n'est pas un
/// avertissement, la géométrie fournie est physiquement invalide.
#[derive(Clone, Debug)]
pub struct GeometryInconsistency {
    pub splay_deg: f64,
    pub lever_mm: f64,
    pub lever_check_mm: f64,
    pub discrepancy_mm: f64,
    /// Renseigné quand l'incohérence vient de la barre arrière et non d'un trou
    /// de couronne : les quatre champs ci-dessus n'ont alors pas de sens.
    pub bar_reason: Option<String>,
}

/// Valeurs dérivées d'une `SpeakerModel`, affichées en lecture seule sur la
/// page "Équipement et enceinte" (brief §9). Échoue si un trou de la grille
/// percée ne recoupe pas la formule trigonométrique à `LEVER_TOLERANCE_MM` près.
pub fn geometry_report(
    speaker: &SpeakerModel,
) -> Result<SpeakerGeometryReport, GeometryInconsistency> {
    let geo = SpeakerGeometry::compute(speaker);
    let bar_warnings = check_rear_bar(speaker).map_err(|e| GeometryInconsistency {
        splay_deg: f64::NAN,
        lever_mm: f64::NAN,
        lever_check_mm: f64::NAN,
        discrepancy_mm: f64::NAN,
        bar_reason: Some(e.reason),
    })?;
    let mut holes = Vec::with_capacity(speaker.mechanical.splay_grid.len());
    for &s in &speaker.mechanical.splay_grid {
        let lever_mm = geo.lever(s);
        let lever_check_mm = geo.lever_check(s);
        let discrepancy_mm = (lever_mm - lever_check_mm).abs();
        if discrepancy_mm > LEVER_TOLERANCE_MM {
            return Err(GeometryInconsistency {
                splay_deg: s,
                lever_mm,
                lever_check_mm,
                discrepancy_mm,
                bar_reason: None,
            });
        }
        holes.push(CrownHoleReport {
            splay_deg: s,
            row: CrownRow::of(s),
            radius: geo.crown_radius_at(s),
            position: geo.crown(s),
            pv: geo.pv_at(s),
            offset: geo.joint_offset(s),
            lever_mm,
            lever_check_mm,
            discrepancy_mm,
        });
    }
    Ok(SpeakerGeometryReport {
        ha: geo.ha,
        ht: geo.ht,
        hb: geo.hb,
        pv0: geo.pv_at(0.0),
        anchor_local: geo.anchor_local,
        latch_local: geo.latch_local,
        front_edge: geo.front_edge,
        bielle_entraxe: geo.bielle_entraxe,
        bar_warnings,
        holes,
        outline: speaker_outline(speaker),
    })
}

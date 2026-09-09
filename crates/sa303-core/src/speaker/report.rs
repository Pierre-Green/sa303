//! Rapport de géométrie d'une `SpeakerModel` (brief §3, §9) : recoupe le bras
//! de levier de chaque trou de la grille percée contre la formule
//! trigonométrique de contrôle, et échoue si l'écart dépasse la tolérance —
//! une géométrie qui ne recoupe pas est physiquement invalide, pas un simple
//! avertissement. Sert la page "Équipement et enceinte", en lecture seule
//! côté front.

use super::geometry::{speaker_outline, CrownRow, SpeakerGeometry};
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
    pub pv: Vec2,
    pub anchor_local: Vec2,
    pub bielle_entraxe: f64,
    /// Un par trou percé de `splay_grid`.
    pub holes: Vec<CrownHoleReport>,
    /// Silhouette de l'enceinte (trapèze), repère enceinte — pour
    /// l'`ArrayViewer` uniquement, afin qu'il n'ait aucune trigonométrie à
    /// faire lui-même.
    pub outline: [Vec2; 4],
}

/// Incohérence de recoupement sur un trou donné (brief §3) : ce n'est pas un
/// avertissement, la géométrie fournie est physiquement invalide.
#[derive(Clone, Copy, Debug)]
pub struct GeometryInconsistency {
    pub splay_deg: f64,
    pub lever_mm: f64,
    pub lever_check_mm: f64,
    pub discrepancy_mm: f64,
}

/// Valeurs dérivées d'une `SpeakerModel`, affichées en lecture seule sur la
/// page "Équipement et enceinte" (brief §9). Échoue si un trou de la grille
/// percée ne recoupe pas la formule trigonométrique à `LEVER_TOLERANCE_MM` près.
pub fn geometry_report(
    speaker: &SpeakerModel,
) -> Result<SpeakerGeometryReport, GeometryInconsistency> {
    let geo = SpeakerGeometry::compute(speaker);
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
            });
        }
        holes.push(CrownHoleReport {
            splay_deg: s,
            row: CrownRow::of(s),
            radius: geo.crown_radius_at(s),
            position: geo.crown(s),
            lever_mm,
            lever_check_mm,
            discrepancy_mm,
        });
    }
    Ok(SpeakerGeometryReport {
        ha: geo.ha,
        ht: geo.ht,
        hb: geo.hb,
        pv: geo.pv,
        anchor_local: geo.anchor_local,
        bielle_entraxe: geo.bielle_entraxe,
        holes,
        outline: speaker_outline(speaker),
    })
}

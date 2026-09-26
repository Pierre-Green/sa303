//! Choix des points d'accroche en vol parmi les trous **réellement percés** du
//! bumper et de sa barre (`crate::bumper::BumperRigging`,
//! `crate::bumper::BumperBarGeometry`).
//!
//! * **1 point** : l'assiette découle du trou (pendaison libre). Le solveur
//!   prend le trou qui s'approche le plus de l'assiette visée ; l'écart
//!   restant est affiché, jamais masqué.
//! * **2 points** : l'assiette est tenue par les longueurs de chaîne, donc
//!   n'importe laquelle est atteignable ; le choix des deux trous ne sert qu'à
//!   répartir la charge. Le solveur prend la paire la plus équilibrée.
//!
//! Dans les deux cas, en mode `Auto`, le bumper seul passe avant la barre, et
//! les montages de barre du plus centré au plus déporté.

use serde::Serialize;

use super::model::{RiggingRequest, RiggingSupport};
use super::solver::ImpossibleConfiguration;
use crate::bumper::{bar_mounts, BarMount, BumperBarModel, BumperModel};
use crate::vector::Vec2;

/// Écart d'assiette en deçà duquel une famille d'accroche est jugée suffisante
/// en mode `Auto`, degrés. Au-delà, on passe à la famille suivante (barre) si
/// elle fait mieux.
pub(super) const TILT_TOLERANCE_DEG: f64 = 0.5;

/// D'où vient un groupe de trous : le bumper, ou la barre dans un montage.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum GroupKind {
    Bumper,
    Bar { mount_index: usize },
}

#[derive(Clone, Debug)]
pub(super) struct Candidate {
    pub label: String,
    /// Repère de l'enceinte du haut.
    pub local: Vec2,
    /// Abscisse dans le repère bumper, mm (positif vers l'arrière).
    pub bumper_x_mm: f64,
    pub wll_kg: f64,
}

#[derive(Clone, Debug)]
pub(super) struct Group {
    pub kind: GroupKind,
    pub candidates: Vec<Candidate>,
}

/// Tout ce que le bumper et sa barre offrent, déjà filtré par la demande de
/// l'utilisateur et trié par ordre de préférence.
pub(super) struct RiggingOptions {
    pub groups: Vec<Group>,
    pub mounts: Vec<BarMount>,
    /// Trous de manille du bumper, repère enceinte du haut — pour le dessin.
    pub bumper_holes_local: Vec<Vec2>,
    /// Trous de liaison de la barre, repère enceinte du haut — pour le dessin,
    /// tous, qu'une patte y soit goupillée ou non.
    pub link_holes_local: Vec<Vec2>,
    /// Décalage repère bumper → repère enceinte du haut, mm.
    pub speaker_half_height: f64,
    /// Trou de manille le plus centré côté arrière, repère enceinte du haut :
    /// référence de « φ libre ». `None` si le bumper n'a pas de manille arrière.
    pub reference_hole_local: Option<Vec2>,
}

fn side_label(x: f64) -> &'static str {
    if x < 0.0 {
        "AV"
    } else {
        "AR"
    }
}

/// Les trous que le bumper et sa barre offrent pour cette demande. Un bumper
/// sans aucun trou utilisable pour elle est une configuration impossible.
pub(super) fn rigging_options(
    bumper: &BumperModel,
    bar: Option<&BumperBarModel>,
    speaker_half_height: f64,
    request: &RiggingRequest,
) -> Result<RiggingOptions, ImpossibleConfiguration> {
    let rigging = &bumper.rigging;
    let to_speaker = |p: Vec2| Vec2::new(p.x, p.y + speaker_half_height);

    let bumper_holes: Vec<Vec2> = rigging
        .shackle_holes
        .iter()
        .map(|&[x, y]| Vec2::new(x, y))
        .collect();
    // Numérotés du centre vers l'extérieur, de chaque côté : « AR1 » est le
    // trou arrière le plus proche du centre.
    let bumper_candidates = bumper_holes
        .iter()
        .map(|&p| {
            let rank = bumper_holes
                .iter()
                .filter(|q| q.x.signum() == p.x.signum() && q.x.abs() < p.x.abs())
                .count();
            Candidate {
                label: format!("Bumper {}{}", side_label(p.x), rank + 1),
                local: to_speaker(p),
                bumper_x_mm: p.x,
                wll_kg: rigging.wll_kg,
            }
        })
        .collect();

    let bar_geometry = bar.map(|b| &b.geometry);
    let mounts = bar_geometry
        .map(|g| bar_mounts(rigging, g))
        .unwrap_or_default();

    let mut groups = Vec::new();
    if request.support != RiggingSupport::Bar {
        groups.push(Group {
            kind: GroupKind::Bumper,
            candidates: bumper_candidates,
        });
    }
    if request.support != RiggingSupport::Bumper {
        if let Some(geometry) = bar_geometry {
            let centre = geometry.pickup_holes.len() / 2;
            for (mount_index, mount) in mounts.iter().enumerate() {
                if request.bar_mount_index.is_some_and(|i| i != mount_index) {
                    continue;
                }
                let candidates = geometry
                    .pickup_holes
                    .iter()
                    .enumerate()
                    .map(|(i, &p)| {
                        let q = mount.to_bumper(p);
                        // Numéro compté depuis le centre de la barre, positif
                        // vers l'arrière quel que soit le sens de montage.
                        let rank = i as i64 - centre as i64;
                        let rank = if mount.flipped { -rank } else { rank };
                        Candidate {
                            label: format!("Barre {} trou {rank:+}", mount_label(mount)),
                            local: to_speaker(q),
                            bumper_x_mm: q.x,
                            wll_kg: geometry.wll_kg,
                        }
                    })
                    .collect();
                groups.push(Group {
                    kind: GroupKind::Bar { mount_index },
                    candidates,
                });
            }
        }
    }

    groups.retain(|g| !g.candidates.is_empty());
    if groups.is_empty() {
        let reason = match request.support {
            RiggingSupport::Bar if bar_geometry.is_none() => format!(
                "Aucune barre de déport cotée n'est compatible avec le bumper \"{}\".",
                bumper.name
            ),
            RiggingSupport::Bar => "Le montage de barre demandé n'existe pas avec ces perçages.".into(),
            _ => format!("Le bumper \"{}\" ne déclare aucun trou d'accroche.", bumper.name),
        };
        return Err(ImpossibleConfiguration { reason });
    }

    // Référence de « φ libre » : le trou de manille le plus centré côté
    // arrière, où la grappe penche naturellement vers l'avant — ou le trou
    // central, s'il y en a un.
    let reference_hole_local = bumper_holes
        .iter()
        .filter(|p| p.x >= 0.0)
        .min_by(|a, b| a.x.total_cmp(&b.x))
        .map(|&p| to_speaker(p));

    Ok(RiggingOptions {
        groups,
        mounts,
        bumper_holes_local: bumper_holes.into_iter().map(to_speaker).collect(),
        link_holes_local: rigging
            .bar_link_holes
            .iter()
            .map(|&[x, y]| to_speaker(Vec2::new(x, y)))
            .collect(),
        speaker_half_height,
        reference_hole_local,
    })
}

pub(super) fn mount_label(mount: &BarMount) -> String {
    format!(
        "{} {:.0} mm",
        side_label(mount.center_x_mm),
        mount.center_x_mm.abs()
    )
}

/// Accroche retenue à un point : le trou, et l'assiette qu'il donne en
/// pendaison libre (rad).
pub(super) struct SinglePick {
    pub group: GroupKind,
    pub candidate: Candidate,
    pub phi: f64,
}

/// Le meilleur trou pour viser `target` (rad), ou, sans assiette visée, le trou
/// qui laisse la grappe la plus proche de l'horizontale. `phi_of` rend
/// l'assiette de pendaison libre depuis un point du repère enceinte.
///
/// Famille par famille dans l'ordre de préférence : la première qui tombe dans
/// `TILT_TOLERANCE_DEG` l'emporte, sinon le meilleur trou toutes familles
/// confondues.
pub(super) fn pick_single(
    options: &RiggingOptions,
    target: Option<f64>,
    phi_of: impl Fn(Vec2) -> f64,
) -> SinglePick {
    let aim = target.unwrap_or(0.0);
    let tolerance = TILT_TOLERANCE_DEG.to_radians();
    let mut best: Option<(f64, SinglePick)> = None;
    for group in &options.groups {
        let mut group_best: Option<(f64, SinglePick)> = None;
        for candidate in &group.candidates {
            let phi = phi_of(candidate.local);
            let err = (phi - aim).abs();
            if group_best.as_ref().is_none_or(|(e, _)| err < *e) {
                group_best = Some((
                    err,
                    SinglePick {
                        group: group.kind,
                        candidate: candidate.clone(),
                        phi,
                    },
                ));
            }
        }
        let Some((err, pick)) = group_best else { continue };
        if err <= tolerance {
            return pick;
        }
        if best.as_ref().is_none_or(|(e, _)| err < *e) {
            best = Some((err, pick));
        }
    }
    best.expect("rigging_options garantit au moins un trou").1
}

/// Plage d'assiettes atteignables à un point (rad), toutes familles permises.
pub(super) fn single_tilt_range(options: &RiggingOptions, phi_of: impl Fn(Vec2) -> f64) -> (f64, f64) {
    options
        .groups
        .iter()
        .flat_map(|g| &g.candidates)
        .map(|c| phi_of(c.local))
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), phi| (lo.min(phi), hi.max(phi)))
}

/// Deux points d'une même famille et la charge verticale de chacun, N.
pub(super) struct PairPick {
    pub group: GroupKind,
    pub points: [(Candidate, f64); 2],
}

/// La paire de trous qui équilibre au mieux les deux moteurs, à assiette
/// fixée : deux chaînes verticales, donc la répartition se lit directement sur
/// les abscisses globales (`to_global_x`) de part et d'autre du CG.
///
/// Famille par famille dans l'ordre de préférence : la première où une paire
/// encadre le CG sans dépasser la charge maximale d'aucun trou l'emporte ;
/// sinon la paire la plus équilibrée toutes familles confondues (l'alerte de
/// surcharge est alors affichée). `None` si aucune paire n'encadre le CG.
pub(super) fn pick_pair(
    options: &RiggingOptions,
    cg_x: f64,
    total_weight_n: f64,
    gravity: f64,
    to_global_x: impl Fn(Vec2) -> f64,
) -> Option<PairPick> {
    let mut best: Option<(f64, PairPick)> = None;
    for group in &options.groups {
        let mut group_best: Option<(f64, PairPick)> = None;
        let c = &group.candidates;
        for i in 0..c.len() {
            for j in (i + 1)..c.len() {
                let (xa, xb) = (to_global_x(c[i].local), to_global_x(c[j].local));
                let span = xb - xa;
                if span.abs() < 1e-6 {
                    continue;
                }
                let ta = total_weight_n * (xb - cg_x) / span;
                let tb = total_weight_n - ta;
                if ta <= 0.0 || tb <= 0.0 {
                    continue;
                }
                // Le plus chargé des deux, à minimiser ; à égalité, la paire la
                // plus écartée, plus stable.
                let score = ta.max(tb) - span.abs() * 1e-9;
                if group_best.as_ref().is_none_or(|(s, _)| score < *s) {
                    group_best = Some((
                        score,
                        PairPick {
                            group: group.kind,
                            points: [(c[i].clone(), ta), (c[j].clone(), tb)],
                        },
                    ));
                }
            }
        }
        let Some((score, pick)) = group_best else { continue };
        let within_wll = pick
            .points
            .iter()
            .all(|(c, t)| t / gravity <= c.wll_kg);
        if within_wll {
            return Some(pick);
        }
        if best.as_ref().is_none_or(|(s, _)| score < *s) {
            best = Some((score, pick));
        }
    }
    best.map(|(_, p)| p)
}

/// Un point d'accroche retenu, tel qu'affiché.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiggingPointView {
    /// « Bumper AR2 », « Barre AR 258 mm trou +3 »…
    pub label: String,
    /// Abscisse dans le repère bumper, mm (positif vers l'arrière).
    pub bumper_x_mm: f64,
    pub point_global: Vec2,
    /// Charge dans la chaîne, N (poids × k_dyn, comme les autres efforts).
    pub tension_n: f64,
    /// La même, en kg.
    pub load_kg: f64,
    pub wll_kg: f64,
    pub overloaded: bool,
}

/// Accroche retenue en vol, quand le bumper déclare ses trous.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RiggingView {
    /// Famille réellement retenue : `Bumper` ou `Bar`, jamais `Auto`.
    pub support: RiggingSupport,
    /// Tous les montages de barre possibles, pour le sélecteur.
    pub bar_mounts: Vec<BarMountView>,
    /// Montage retenu, indice dans `bar_mounts`.
    pub bar_mount_index: Option<usize>,
    pub points: Vec<RiggingPointView>,
    pub target_tilt_deg: Option<f64>,
    pub achieved_tilt_deg: f64,
    /// Écart entre l'assiette obtenue et l'assiette visée, degrés. Non nul
    /// seulement à un point, sans pull-back.
    pub tilt_error_deg: Option<f64>,
    pub bumper_holes_global: Vec<Vec2>,
    /// Les trous de liaison de la barre sur le bumper, tous.
    pub bumper_link_holes_global: Vec<Vec2>,
    /// Barre montée seulement.
    pub bar_holes_global: Vec<Vec2>,
    pub bar_pins_global: Vec<Vec2>,
    /// Silhouette schématique de la barre montée, polygone fermé.
    pub bar_outline_global: Vec<Vec2>,
    /// Ce que chaque patte de la barre transmet au trou de liaison du bumper,
    /// dans l'ordre de `bar_pins_global`. Barre montée seulement.
    pub bar_link_forces: Vec<LinkForceView>,
}

/// Effort à une goupille de liaison barre / bumper. Charge entière de la
/// barre, pas par flanc.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkForceView {
    pub point_global: Vec2,
    pub force_global: Vec2,
    pub force_n: f64,
    /// Direction dans le repère de l'enceinte du haut (convention §2), comme
    /// les autres efforts du bumper.
    pub angle_deg: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BarMountView {
    pub label: String,
    pub flipped: bool,
    pub center_x_mm: f64,
}

impl From<&BarMount> for BarMountView {
    fn from(m: &BarMount) -> Self {
        Self {
            label: mount_label(m),
            flipped: m.flipped,
            center_x_mm: m.center_x_mm,
        }
    }
}

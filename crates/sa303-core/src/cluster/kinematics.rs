//! Cinématique de grappe (brief §4) : position, inclinaison et CG global de
//! chaque enceinte une fois la chaîne assemblée, les deux façons de fixer
//! `φ_initial` (pendaison libre, ou assiette de calage en stack), et
//! l'inversion utilisée pour dériver le point d'accroche nécessaire à une
//! assiette imposée.
//!
//! Une grappe est **hétérogène** : chaque position porte son propre modèle
//! d'enceinte (SA303-ISOPHASE, SA303-CCA, renfort de grave, ...), donc sa
//! propre géométrie, masse et CG. Tout le calcul travaille sur une chaîne de
//! `ChainSpeaker` — un bundle `Copy` dérivé une seule fois par position — au
//! lieu d'une géométrie unique partagée : c'est ce qui évite de trimballer des
//! tableaux parallèles (masses, CG, silhouettes) dans chaque signature.

use crate::speaker::{SpeakerGeometry, SpeakerModel};
use crate::vector::Vec2;
use serde::Serialize;

/// Une enceinte de la chaîne, entièrement dérivée de son `SpeakerModel` :
/// géométrie (trous, couronne), masse, CG local et silhouette. Tout ce dont la
/// physique a besoin pour une position donnée, calculé une fois pour toutes.
#[derive(Clone, Copy, Debug)]
pub struct ChainSpeaker {
    pub geo: SpeakerGeometry,
    pub cg_local: Vec2,
    pub mass_kg: f64,
    pub frame_hole_splay: f64,
    /// Silhouette (trapèze) en repère enceinte — sert au test de collision de
    /// la tirette et au rendu, sans jamais recalculer la trigonométrie.
    pub outline: [Vec2; 4],
}

impl ChainSpeaker {
    pub fn from_model(model: &SpeakerModel) -> Self {
        let m = &model.mechanical;
        Self {
            geo: SpeakerGeometry::compute(model),
            cg_local: Vec2::new(m.cg[0], m.cg[1]),
            mass_kg: m.mass_kg,
            frame_hole_splay: m.frame_hole_splay,
            outline: crate::speaker::speaker_outline(model),
        }
    }
}

/// Position, inclinaison et CG global d'une enceinte dans une grappe. Le CG
/// global et la silhouette sont fournis ici (plutôt que recalculés côté front)
/// pour que l'`ArrayViewer` n'ait jamais à faire de rotation lui-même — seule
/// la mise à l'échelle pixels est tolérée en TypeScript (brief §1). La
/// silhouette est par enceinte : dans une grappe hétérogène, deux positions
/// n'ont pas forcément la même.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpeakerInstance {
    pub o: Vec2,
    pub phi: f64,
    pub cg: Vec2,
    /// Silhouette (trapèze) en repère enceinte, à dessiner telle quelle dans
    /// le groupe déjà tourné de cette enceinte.
    pub outline: [Vec2; 4],
}

/// Construit la chaîne d'enceintes (brief §4). `splays_deg` du haut vers le
/// bas, `chain` compte exactement une enceinte de plus.
///
/// La jonction relie le pivot effectif de l'enceinte du **haut** (`pv`, qui
/// matérialise où atterrit le trou avant-haut de celle du dessous) à la
/// charnière haute de l'enceinte du **bas** (`ht`) : d'où les deux géométries
/// différentes dans le même pas. Déclarer deux modèles compatibles
/// (`BelowCompatibility`), c'est justement affirmer que ces deux pièces
/// s'assemblent.
pub fn build_cluster(
    chain: &[ChainSpeaker],
    splays_deg: &[f64],
    phi_initial: f64,
) -> Vec<SpeakerInstance> {
    let mut speakers = Vec::with_capacity(chain.len());
    let mut o = Vec2::ZERO;
    let mut phi = phi_initial;
    for (i, speaker) in chain.iter().enumerate() {
        speakers.push(SpeakerInstance {
            o,
            phi,
            cg: o + speaker.cg_local.rotate(phi),
            outline: speaker.outline,
        });
        if i < splays_deg.len() {
            let phi_next = phi + splays_deg[i].to_radians();
            o = (o + speaker.geo.pv.rotate(phi)) - chain[i + 1].geo.ht.rotate(phi_next);
            phi = phi_next;
        }
    }
    speakers
}

/// Barycentre pondéré par la masse de chaque enceinte : dans une grappe
/// hétérogène, un renfort de grave ne pèse pas comme une tête, donc jamais une
/// moyenne simple des positions.
pub fn weighted_cg(chain: &[ChainSpeaker], speakers: &[SpeakerInstance]) -> Vec2 {
    let mut total = 0.0;
    let mut sum = Vec2::ZERO;
    for (speaker, instance) in chain.iter().zip(speakers) {
        total += speaker.mass_kg;
        sum = sum + instance.cg * speaker.mass_kg;
    }
    if total == 0.0 {
        return Vec2::ZERO;
    }
    sum * (1.0 / total)
}

/// `φ_initial` en suspension libre : le CG de l'ensemble passe sous le point de levage.
pub fn phi_initial_free_hang(chain: &[ChainSpeaker], splays_deg: &[f64], pickup: Vec2) -> f64 {
    let speakers = build_cluster(chain, splays_deg, 0.0);
    let cm = weighted_cg(chain, &speakers);
    let d = cm - (speakers[0].o + pickup);
    -(d.x).atan2(-d.y)
}

/// `φ_initial` en stack : fixé par l'angle de l'enceinte du bas.
pub fn phi_initial_stack(bottom_angle_deg: f64, splays_deg: &[f64]) -> f64 {
    let sum_splay_deg: f64 = splays_deg.iter().sum();
    (bottom_angle_deg - sum_splay_deg).to_radians()
}

/// Position d'accroche (x, le long de la barre de déport) nécessaire pour
/// qu'une grappe suspendue sans tirette adopte exactement `phi_initial` à la
/// hauteur d'accroche donnée — l'inverse de `phi_initial_free_hang` : là on
/// part de l'assiette voulue pour remonter au point d'accroche, plutôt que
/// l'inverse.
pub fn solve_pickup_x_for_imposed_tilt(
    chain: &[ChainSpeaker],
    splays_deg: &[f64],
    phi_initial: f64,
    pickup_height: f64,
) -> f64 {
    let speakers = build_cluster(chain, splays_deg, phi_initial);
    let cm = weighted_cg(chain, &speakers);
    let (s, c) = phi_initial.sin_cos();
    (cm.x + pickup_height * s) / c
}

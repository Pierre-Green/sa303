//! Statique d'une jonction (brief §3) : équilibre du corps libre porté par le
//! flanc chargé, sur les deux seules liaisons entre les deux caissons.
//!
//! **La barre arrière n'est plus un élément à deux forces.** Elle est goupillée
//! en *deux* points (ancrage et verrou) dans le caisson du bas, donc encastrée
//! sur lui : elle lui transmet une force et un moment, et sa ligne d'action
//! n'a aucune raison de passer par l'axe couronne-ancrage. C'est la **bielle
//! avant** qui est l'élément à deux forces : goupillée en deux points, sa ligne
//! d'action passe par ses deux goupilles, donc elle s'incline de `splay/2`.
//!
//! Les deux rôles ont donc échangé par rapport au modèle précédent : l'inconnue
//! scalaire est portée par la bielle (direction connue), et l'inconnue
//! vectorielle par la goupille de couronne (moment nul, articulation simple).
//! Trois inconnues, trois équations : isostatique.
//!
//! Le résultat inclut déjà les efforts en repère global et les angles
//! pré-calculés (convention §2) : jamais de rotation ni de `atan2` côté front.
//!
//! Grappe hétérogène : la quincaillerie de la jonction appartient à l'enceinte
//! du **haut** (sa bielle, sa couronne, son ancrage), tandis que les trous
//! rendus en repère local appartiennent à l'enceinte **chargée** — celle du
//! haut en vol, celle du bas en stack. Les deux coïncident quand les deux
//! modèles sont identiques, d'où des résultats inchangés sur une grappe
//! homogène.

use super::kinematics::{ChainSpeaker, SpeakerInstance};
use super::model::Compartment;
use crate::speaker::{CrownRow, JointOffset, SpeakerGeometry, SplayRange};
use crate::tie::TieForce;
use crate::vector::{angle_of, Vec2};
use serde::Serialize;

pub struct JointInput<'a> {
    /// Une entrée par enceinte de la chaîne, du haut vers le bas.
    pub chain: &'a [ChainSpeaker],
    pub speakers: &'a [SpeakerInstance],
    pub splays_deg: &'a [f64],
    pub joint_index: usize,
    pub compartment: Compartment,
    pub g: f64,
    pub k_dyn: f64,
    pub share_per_flank: f64,
    /// Tirette active, vol uniquement.
    pub tie: Option<TieForce>,
    /// Splay recommandé pour cette paire de modèles, s'il y en a un déclaré.
    pub recommended_splay: Option<SplayRange>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JointResult {
    pub joint_index: usize,
    pub compartment: Compartment,
    /// Nombre d'enceintes du corps libre.
    pub free_body_count: usize,
    /// Enceinte dont le flanc est chargé, 0-indexée.
    pub loaded_flank: usize,
    /// Inclinaison absolue de l'enceinte porteuse, degrés.
    pub inclination_deg: f64,
    pub splay_deg: f64,
    pub row: CrownRow,
    pub crown_radius: f64,
    /// Bras de levier géométrique couronne-ancrage. Recoupement de perçage
    /// uniquement : la statique ne s'en sert plus (brief §7).
    pub lever_mm: f64,
    /// Bras de levier de la bielle avant autour de la goupille de couronne,
    /// celui qui résout réellement la jonction.
    pub bielle_lever_mm: f64,
    /// Rotation de la bielle avant, degrés : exactement la moitié du splay.
    pub bielle_rotation_deg: f64,
    /// Écartement des coins avant à cette jonction (brief §5). C'est le
    /// `verticalMm` qu'il faut afficher comme espacement entre caissons.
    pub offset: JointOffset,

    /// Positions des trous, repère du flanc chargé.
    pub loaded_orientation_hole: Vec2,
    pub loaded_pivot_hole: Vec2,
    pub constrained_hinge_hole: Vec2,
    /// Stack seulement : trou de couronne contraint (celui du joint en dessous, ou cadre de calage).
    pub constrained_crown_splay: Option<f64>,
    pub constrained_crown_hole: Option<Vec2>,
    pub constrained_crown_row: Option<CrownRow>,
    pub constrained_crown_radius: Option<f64>,
    pub constrained_is_frame: bool,

    /// Efforts et gravité, repère du flanc chargé.
    pub f_orientation: Vec2,
    pub f_pivot: Vec2,
    pub gravity_local: Vec2,
    /// Intensité et direction (convention §2) de `f_orientation`/`f_pivot`,
    /// pré-calculées pour que l'affichage n'ait jamais à faire de `atan2`.
    pub f_orientation_n: f64,
    pub f_orientation_angle_deg: f64,
    pub f_pivot_n: f64,
    pub f_pivot_angle_deg: f64,

    /// Mêmes trous et efforts, repère global — pour l'`ArrayViewer` uniquement :
    /// jamais de rotation côté TypeScript (brief §1).
    pub loaded_orientation_hole_global: Vec2,
    pub loaded_pivot_hole_global: Vec2,
    pub f_orientation_global: Vec2,
    pub f_pivot_global: Vec2,

    pub traction: bool,
    pub hinge_reversed: bool,
    pub bumper_moment_nm: f64,
    /// Moment déversé par la barre arrière dans le caisson du bas, réduit à
    /// l'ancrage (N·m). Nul dans l'ancien modèle à deux forces.
    pub bar_moment_nm: f64,
    pub residual_n: f64,

    /// Splay recommandé entre ces deux modèles, s'il y en a un déclaré
    /// (`BelowCompatibility::recommended_splay`), en degrés `[min, max]`.
    pub recommended_splay_range_deg: Option<[f64; 2]>,
    /// `false` uniquement si une recommandation existe et que le splay retenu
    /// en sort. Ce n'est **jamais** une erreur : la jonction reste
    /// mécaniquement valable, elle est seulement signalée comme non optimale
    /// acoustiquement. Sans recommandation déclarée : toujours `true`.
    pub acoustically_optimal: bool,
}

impl JointResult {
    pub fn mag_orientation(&self) -> f64 {
        self.f_orientation.norm()
    }
    pub fn mag_pivot(&self) -> f64 {
        self.f_pivot.norm()
    }
}

pub fn compute_joint(input: &JointInput) -> JointResult {
    let i = input.joint_index;
    let n = input.speakers.len();
    let si = input.speakers[i];
    let s = input.splays_deg[i];

    // Quincaillerie de la jonction : elle appartient à l'enceinte du haut.
    let geo = input.chain[i].geo;

    // Les quatre points de la jonction, repère global. `pa`/`pb` sont les deux
    // goupilles de la bielle avant, `bo` la goupille de couronne (articulation
    // simple), `an` l'ancrage de la barre sur le caisson du bas.
    let pa = si.o + geo.hb.rotate(si.phi);
    let pb = si.o + geo.pv_at(s).rotate(si.phi);
    let bo = si.o + geo.crown(s).rotate(si.phi);
    let an = si.o + geo.anchor_at(s).rotate(si.phi);

    let (lo, hi) = match input.compartment {
        Compartment::Flown => (i + 1, n - 1),
        Compartment::Stacked => (0, i),
    };
    // Poids enceinte par enceinte : dans une grappe hétérogène, un renfort de
    // grave ne pèse pas comme une tête — jamais une masse unique multipliée.
    let mut w_total = 0.0;
    let mut sum = Vec2::ZERO;
    for k in lo..=hi {
        let w = input.chain[k].mass_kg * input.g * input.k_dyn;
        w_total += w;
        sum = sum + input.speakers[k].cg * w;
    }
    let cm = sum * (1.0 / w_total);

    // Moment extérieur pris à la goupille de couronne : c'est elle qui porte
    // l'inconnue vectorielle, donc c'est en ce point qu'elle disparaît de
    // l'équation de moment.
    let mut rext = Vec2::new(0.0, -w_total);
    let mut mext = (cm - bo).cross(rext);

    if let (Compartment::Flown, Some(tie)) = (input.compartment, input.tie) {
        let last = input.speakers[n - 1];
        let q = last.o + tie.point_local.rotate(last.phi);
        rext = rext + tie.force;
        mext += (q - bo).cross(tie.force);
    }

    // Bielle avant, élément à deux forces : direction imposée par ses deux
    // goupilles, donc une seule inconnue scalaire.
    let u = (pb - pa).normalize();
    let bielle_lever = (pb - bo).cross(u);
    let lambda = -mext / bielle_lever;
    let f_piv = u * lambda;
    // La goupille de couronne reprend tout le reste : c'est par elle que la
    // barre arrière, encastrée sur le caisson du bas, passe son effort.
    let f_ori = -rext - f_piv;

    let ti = match input.compartment {
        Compartment::Flown => i,
        Compartment::Stacked => i + 1,
    };
    // Trous exprimés dans le repère du flanc chargé : c'est donc la géométrie
    // de CETTE enceinte-là (celle du haut en vol, celle du bas en stack).
    let loaded = input.chain[ti];
    let rt_phi = input.speakers[ti].phi;
    let sg = -input.share_per_flank;

    let f_orientation = (f_ori * sg).rotate_transpose(rt_phi);
    let f_pivot = (f_piv * sg).rotate_transpose(rt_phi);
    let gravity_local = Vec2::new(0.0, -1.0).rotate_transpose(rt_phi);
    let ext_local = (rext * input.share_per_flank).rotate_transpose(rt_phi);
    let residual = (f_orientation + f_pivot - ext_local).norm();

    let (loaded_orientation_hole, constrained_crown_splay) = match input.compartment {
        Compartment::Flown => (loaded.geo.crown(s), None),
        Compartment::Stacked => {
            let below = if ti < input.splays_deg.len() {
                Some(input.splays_deg[ti])
            } else {
                None
            };
            (
                loaded.geo.anchor_local,
                Some(below.unwrap_or(loaded.frame_hole_splay)),
            )
        }
    };
    let loaded_pivot_hole = match input.compartment {
        Compartment::Flown => loaded.geo.hb,
        Compartment::Stacked => loaded.geo.ht,
    };
    let constrained_hinge_hole = match input.compartment {
        Compartment::Flown => loaded.geo.ht,
        Compartment::Stacked => loaded.geo.hb,
    };
    let constrained_is_frame =
        matches!(input.compartment, Compartment::Stacked) && ti >= input.splays_deg.len();
    let constrained_crown_hole = constrained_crown_splay.map(|cs| loaded.geo.crown(cs));
    let constrained_crown_row = constrained_crown_splay.map(CrownRow::of);
    let constrained_crown_radius = constrained_crown_splay.map(|cs| loaded.geo.crown_radius_at(cs));

    let tg = input.speakers[ti];
    let loaded_orientation_hole_global = tg.o + loaded_orientation_hole.rotate(tg.phi);
    let loaded_pivot_hole_global = tg.o + loaded_pivot_hole.rotate(tg.phi);
    let f_orientation_global = f_ori * sg;
    let f_pivot_global = f_piv * sg;

    let f_orientation_n = f_orientation.norm();
    let f_orientation_angle_deg = angle_of(f_orientation);
    let f_pivot_n = f_pivot.norm();
    let f_pivot_angle_deg = angle_of(f_pivot);

    let hinge_reversed = f_pivot.dot(gravity_local) < 0.0;
    // Entraxe de bielle de l'enceinte du haut : c'est elle qui porte le bras
    // entre `hb` et la goupille basse de la jonction.
    let bumper_moment_nm = f_pivot.norm() * geo.bielle_entraxe / 1000.0;
    // Moment que la barre arrière, encastrée sur le caisson du bas, y déverse
    // en plus de sa force — réduit à l'ancrage. C'est exactement ce que le
    // modèle à deux forces ignorait : il le supposait nul.
    //
    // Sa répartition entre l'ancrage et le verrou reste indéterminée : deux
    // goupilles dans un même caisson, ce sont six inconnues pour trois
    // équations. Il faut une hypothèse de groupe de goupilles, que le brief ne
    // fixe pas — d'où la seule résultante ici, à l'ancrage.
    let bar_moment_nm = ((bo - an).cross(f_ori) * input.share_per_flank).abs() / 1000.0;
    // Traction de la barre : composante de l'effort de couronne le long de son
    // axe. Le scalaire `lambda` ne renseigne plus là-dessus — il porte
    // désormais la bielle, pas la barre.
    let bar_axial = f_ori.dot((bo - an).normalize());
    let traction = match input.compartment {
        Compartment::Stacked => bar_axial <= 0.0,
        Compartment::Flown => bar_axial >= 0.0,
    };
    let free_body_count = match input.compartment {
        Compartment::Flown => n - i - 1,
        Compartment::Stacked => i + 1,
    };

    JointResult {
        joint_index: i,
        compartment: input.compartment,
        free_body_count,
        loaded_flank: ti,
        inclination_deg: rt_phi.to_degrees(),
        splay_deg: s,
        row: CrownRow::of(s),
        crown_radius: geo.crown_radius_at(s),
        lever_mm: geo.lever(s),
        bielle_lever_mm: bielle_lever.abs(),
        bielle_rotation_deg: SpeakerGeometry::bielle_rotation_deg(s),
        offset: geo.joint_offset(s),
        loaded_orientation_hole,
        loaded_pivot_hole,
        constrained_hinge_hole,
        constrained_crown_splay,
        constrained_crown_hole,
        constrained_crown_row,
        constrained_crown_radius,
        constrained_is_frame,
        f_orientation,
        f_pivot,
        gravity_local,
        f_orientation_n,
        f_orientation_angle_deg,
        f_pivot_n,
        f_pivot_angle_deg,
        loaded_orientation_hole_global,
        loaded_pivot_hole_global,
        f_orientation_global,
        f_pivot_global,
        traction,
        hinge_reversed,
        bumper_moment_nm,
        bar_moment_nm,
        residual_n: residual,
        recommended_splay_range_deg: input.recommended_splay.map(|r| [r.min_deg, r.max_deg]),
        acoustically_optimal: input.recommended_splay.is_none_or(|r| r.contains(s)),
    }
}

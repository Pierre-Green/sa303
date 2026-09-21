//! Assemblage d'une grappe ou d'un stack entier et décision du solveur
//! (brief §4, §5) : dérive le point d'accroche depuis le bumper obligatoire,
//! décide si une barre puis une tirette sont nécessaires, calcule chaque
//! jonction, et rend le tout exploitable par le front (`ClusterResult`) sans
//! qu'il ait jamais à recalculer quoi que ce soit.

use super::joint::{compute_joint, JointInput, JointResult};
use super::kinematics::{
    build_cluster, phi_initial_free_hang, phi_initial_stack, solve_pickup_x_for_imposed_tilt,
    weighted_cg, ChainSpeaker, SpeakerInstance,
};
use super::model::{Cluster, Compartment};
use super::pair::{split_over_pair, split_wrench_over_pair};
use crate::bumper::{
    bumper_outline_top, bumper_pickup_height, bumper_pin_points, BumperBarModel, BumperModel,
};
use crate::settings::Settings;
use crate::speaker::{SpeakerModel, SplayRange};
use crate::tie::{tie_default_angle_deg, tie_tension, tie_valid_angle_range_deg, TieForce};
use crate::vector::{angle_of, dir_from_angle, ray_hits_polygon, Vec2};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterResult {
    pub speakers: Vec<SpeakerInstance>,
    pub phi_initial: f64,
    /// Angle qu'adopterait la grappe en pendaison libre (accroche centrée sur
    /// le bumper), toujours calculé en vol, indépendamment d'une éventuelle
    /// assiette imposée — sert de référence de comparaison, jamais remplacé
    /// par `phi_initial` (brief, correction utilisateur : les deux doivent
    /// rester visibles séparément). `None` en stack, où la notion de
    /// pendaison libre n'a pas de sens.
    pub phi_free_hang: Option<f64>,
    pub tie_tension_n: f64,
    pub joints: Vec<JointResult>,
    /// CG de l'ensemble, repère global : barycentre **pondéré par la masse**
    /// de chaque enceinte — dans une grappe hétérogène, un renfort de grave ne
    /// pèse pas comme une tête.
    pub cg: Vec2,
    /// Masse totale de la chaîne, kg — somme des enceintes réellement montées,
    /// pour que le front n'ait jamais à multiplier une masse par un compte.
    pub total_mass_kg: f64,
    /// Nom du modèle monté à chaque position, même ordre que `speakers` —
    /// purement pour l'affichage (popup du viewer), le front n'a pas à
    /// re-résoudre les ids contre le catalogue.
    pub speaker_names: Vec<String>,
    /// Point de levage, repère global. Vol uniquement.
    pub pickup_global: Option<Vec2>,
    /// Point d'accroche de la tirette sur l'enceinte du bas, repère global.
    pub tie_point_global: Option<Vec2>,
    /// Direction de traction de la tirette, repère global (convention §2).
    pub tie_direction_global: Option<Vec2>,
    /// Même direction, en degrés (convention §2) — pour affichage : le
    /// rigger doit connaître l'angle réel auquel ancrer la tirette. Choisie
    /// par l'utilisateur, ou suggérée par le solveur tant qu'il n'a pas
    /// encore choisi.
    pub tie_direction_angle_deg: Option<f64>,
    /// Bumper attaché à l'enceinte de référence (haut en vol, bas en stack).
    /// Toujours présent : un bumper est obligatoire (brief §11.6).
    pub bumper_view: BumperView,
    /// Position de l'ensemble dans l'espace, dérivée de la hauteur de bumper
    /// déclarée sur la grappe.
    pub elevation: Elevation,
}

/// Altitudes au-dessus du sol, mm. Purement descriptif : aucun effort n'en
/// dépend.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Elevation {
    /// À ajouter à un `y` du repère global pour obtenir une altitude. C'est le
    /// seul champ dont l'écran a besoin pour situer un point quelconque, par
    /// exemple sous le curseur.
    pub offset_mm: f64,
    /// Dessous du bumper : exactement la valeur saisie sur la grappe.
    pub bumper_bottom_mm: f64,
    /// Point le plus bas de l'ensemble, bumper compris — en stack et à hauteur
    /// nulle, c'est le sol.
    pub lowest_point_mm: f64,
    pub highest_point_mm: f64,
    /// Vol uniquement : altitude du point de levage.
    pub pickup_mm: Option<f64>,
    /// Altitude du dessous de chaque enceinte, même ordre que
    /// `ClusterResult::speakers` — c'est la cote qu'un rigger lit au mètre.
    /// Calculée ici plutôt qu'à l'écran : c'est le coin le plus bas de la
    /// silhouette une fois tournée, donc de la trigonométrie (brief §1).
    pub speaker_bottom_mm: Vec<f64>,
}

/// Rendu du bumper : silhouette toujours en repère global (comme les autres
/// champs `*_global`), pour que l'`ArrayViewer` n'ait jamais à le tourner.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BumperView {
    pub outline_global: [Vec2; 4],
    /// Vol uniquement : point d'accroche effectif (sur le bumper, ou sur la
    /// barre de déport si `bar_deport_mm != 0`). Toujours renseigné en vol.
    pub pickup_global: Option<Vec2>,
    /// Départ de la barre de déport sur le bord de la zone de fixation
    /// directe (`max_direct_deport_mm`), repère global. Présent seulement si
    /// une barre est nécessaire.
    pub bumper_bar_start_global: Option<Vec2>,
    /// Position de l'accroche **par rapport au centre du bumper** (mm signés,
    /// axe x de l'enceinte : positif vers l'arrière). C'est la cote que le
    /// rigger reporte pour percer ou repérer sa manille, donc elle est non
    /// nulle dès que l'accroche n'est pas centrée — y compris quand elle reste
    /// sur le bumper. Toujours renseignée en vol.
    pub pickup_offset_mm: Option<f64>,
    /// Ce que la **barre** porte : dépassement signé au-delà de
    /// `max_direct_deport_mm`, donc 0 tant que l'accroche tombe sur le bumper.
    /// À ne pas confondre avec `pickup_offset_mm` : celui-ci décrit où est
    /// l'accroche, celui-là si une barre est nécessaire et de combien.
    /// Toujours renseigné en vol, même quand nul.
    pub bar_deport_mm: Option<f64>,
    /// Au-delà de la portée de la barre (`BumperBarModel::max_deport_mm`), elle ne suffit plus : une
    /// tirette est automatiquement mise en place (voir `tie_tension_n`,
    /// `tie_point_global`).
    pub bumper_bar_exceeded: bool,
    /// Plage de directions de traction physiquement valables pour la tirette
    /// (degrés, convention §2), présente seulement si `bumper_bar_exceeded`. Le
    /// point d'ancrage réel dépend du terrain : à l'utilisateur de choisir
    /// dedans, pas à l'algorithme.
    pub tie_angle_range_deg: Option<[f64; 2]>,
    /// Efforts transmis par le bumper à l'enceinte de référence, repère de
    /// cette enceinte : ce que chacun de ses **deux pions** encaisse.
    /// `orientation` est le pion arrière, `pivot` le pion avant — les noms
    /// datent d'un schéma antérieur, la répartition est aujourd'hui symétrique
    /// entre les deux. Vol uniquement. Par flanc, comme les efforts de
    /// jonction.
    pub orientation_force_n: Option<f64>,
    pub orientation_angle_deg: Option<f64>,
    pub pivot_force_n: Option<f64>,
    pub pivot_angle_deg: Option<f64>,

    /// Les deux mêmes efforts en repère **global**, avec leur point
    /// d'application sur l'enceinte de référence : le viewer les dessine tels
    /// quels, il n'a pas à tourner un vecteur (brief §1).
    /// Les deux pions du bumper, à leur perçage **déclaré**
    /// (`BumperModel::pins`), repère global. Point d'application des deux
    /// efforts ci-dessus : la statique se résout là où la barre est réellement
    /// boulonnée, et c'est le seul endroit d'où ces positions sortent.
    pub orientation_point_global: Option<Vec2>,
    pub pivot_point_global: Option<Vec2>,
    /// La paire ancrage/verrou par laquelle la barre du bumper est boulonnée sur
    /// l'enceinte de référence. Vol uniquement : en stack le bumper est
    /// simplement goupillé, il n'y a pas de barre à encastrer.
    ///
    /// Renseignée pour que l'enceinte de référence montre les mêmes perçages
    /// chargés que n'importe quelle autre enceinte de la grappe — elle est tenue
    /// par la même quincaillerie, ce n'est pas un cas particulier.
    pub pair_anchor_hole_global: Option<Vec2>,
    pub pair_latch_hole_global: Option<Vec2>,
    pub f_pair_anchor_global: Option<Vec2>,
    pub f_pair_latch_global: Option<Vec2>,
    pub f_pair_anchor_n: Option<f64>,
    pub f_pair_anchor_angle_deg: Option<f64>,
    pub f_pair_latch_n: Option<f64>,
    pub f_pair_latch_angle_deg: Option<f64>,
    /// Moment de la barre du bumper au barycentre de la paire, N·m par flanc.
    pub pair_moment_nm: Option<f64>,
    pub orientation_force_global: Option<Vec2>,
    pub pivot_force_global: Option<Vec2>,

    /// Charge que le bumper reprend en entier : la manille en vol, la réaction
    /// du sol en stack. **Pas** par flanc — une manille n'est pas doublée,
    /// contrairement à la quincaillerie de flanc. C'est le poids dynamisé de
    /// toute la grappe, tirette comprise.
    pub support_force_n: f64,
    pub support_force_global: Vec2,
    pub support_point_global: Vec2,
    /// Direction de `support_force_global` (convention §2), pré-calculée.
    pub support_angle_deg: f64,

    /// Moment que la structure du bumper doit transférer **entre ses deux
    /// pions**, réduit à leur milieu (N·m, par flanc). C'est l'équivalent, pour
    /// le bumper, du moment de barre d'une jonction : deux points d'accroche
    /// sur un même corps rigide, donc un couple à passer de l'un à l'autre.
    pub pin_pair_moment_nm: f64,
    /// Entraxe des deux pions, mm — le bras de ce couple.
    pub pin_span_mm: f64,
}

/// Moment que deux efforts de pion imposent à la pièce qui les relie, réduit au
/// milieu des deux. Sert au bumper dans les deux compartiments — qui se
/// résolvent désormais de la même façon, par répartition élastique à raideurs
/// égales — et répond à la même question : que doit encaisser la structure
/// entre ses deux points d'accroche.
fn pin_pair_moment_nm(p1: Vec2, f1: Vec2, p2: Vec2, f2: Vec2) -> f64 {
    let g = (p1 + p2) * 0.5;
    ((p1 - g).cross(f1) + (p2 - g).cross(f2)) / 1000.0
}

/// Ce que `compute_bumper_loads` rend : les deux efforts transmis à l'enceinte
/// de référence, leurs points d'application, et la charge totale reprise par la
/// manille. Un type nommé plutôt qu'un tuple de huit éléments — à ce
/// nombre-là, l'ordre des champs n'est plus lisible sur le site d'appel.
struct BumperLoads {
    orientation_force: Vec2,
    orientation_point: Vec2,
    pivot_force: Vec2,
    pivot_point: Vec2,
    /// La barre du bumper est boulonnée sur l'enceinte de référence par la même
    /// paire de goupilles qu'une jonction ordinaire : ancrage et verrou. Ce que
    /// ces deux goupilles-là subissent, par flanc.
    pair_anchor_point: Vec2,
    pair_latch_point: Vec2,
    pair_anchor_force: Vec2,
    pair_latch_force: Vec2,
    /// Moment de la barre réduit au barycentre de la paire, N·m par flanc.
    pair_moment_nm: f64,
    /// Résultante extérieure reprise par la manille, déjà retournée : c'est ce
    /// que la manille **tire**, pas ce que la grappe pèse.
    support_force: Vec2,
}

/// Chaîne résolue : chaque position de `Cluster::speaker_model_ids` pointe
/// vers un modèle du catalogue, et chaque jonction consécutive doit être
/// déclarée compatible dans ce compartiment. Une référence inconnue ou une
/// jonction non déclarée est une configuration impossible, jamais un
/// assemblage silencieux (brief §11.6).
struct ResolvedChain<'a> {
    models: Vec<&'a SpeakerModel>,
    chain: Vec<ChainSpeaker>,
    /// Une entrée par jonction : le splay recommandé déclaré par l'enceinte du
    /// haut pour celle du bas, s'il y en a un.
    recommended_splays: Vec<Option<SplayRange>>,
}

/// Un trou est percé à cet angle. Tolérance : les splays sont des trous, pas
/// des réels libres — un 10.0 saisi ne doit jamais rater un 10.0 percé pour
/// cause d'arrondi de représentation.
fn has_hole_at(speaker: &SpeakerModel, splay_deg: f64) -> bool {
    speaker
        .mechanical
        .splay_grid
        .iter()
        .any(|&hole| (hole - splay_deg).abs() < 1e-9)
}

fn format_splay_grid(grid: &[f64]) -> String {
    grid.iter()
        .map(|s| format!("{s:.0}°"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn resolve_chain<'a>(
    speaker_models: &'a [SpeakerModel],
    cluster: &Cluster,
) -> Result<ResolvedChain<'a>, ImpossibleConfiguration> {
    let expected = cluster.joints.len() + 1;
    if cluster.speaker_model_ids.len() != expected {
        return Err(ImpossibleConfiguration {
            reason: format!(
                "Grappe incohérente : {} enceinte(s) déclarée(s) pour {} jonction(s), il en faut exactement {expected}.",
                cluster.speaker_model_ids.len(),
                cluster.joints.len()
            ),
        });
    }

    let mut models = Vec::with_capacity(expected);
    for id in &cluster.speaker_model_ids {
        let Some(model) = speaker_models.iter().find(|m| &m.id == id) else {
            return Err(ImpossibleConfiguration {
                reason: format!("Enceinte \"{id}\" introuvable."),
            });
        };
        models.push(model);
    }

    let flown = cluster.compartment == Compartment::Flown;
    let compartment_label = if flown { "en vol" } else { "en stack" };
    let mut recommended_splays = Vec::with_capacity(cluster.joints.len());
    for (upper, lower) in models.iter().zip(models.iter().skip(1)) {
        let Some(compat) = upper.below(&lower.id).filter(|c| c.allows(flown)) else {
            return Err(ImpossibleConfiguration {
                reason: format!(
                    "\"{}\" n'est pas déclarée accrochable sous \"{}\" {compartment_label}. Déclare cette compatibilité dans Équipement et enceinte, ou change d'enceinte à cette position.",
                    lower.name, upper.name
                ),
            });
        };
        recommended_splays.push(compat.recommended_splay);
    }

    // La quincaillerie de jonction appartient à l'enceinte du dessus : c'est sa
    // couronne qui est percée, donc c'est sa grille de trous qui fait foi. Un
    // angle absent de cette grille n'est pas un angle « approché » — il n'a
    // aucun trou où passer la broche, donc la grappe n'est pas montable
    // (brief §11.6).
    for (index, joint) in cluster.joints.iter().enumerate() {
        let upper = models[index];
        if !has_hole_at(upper, joint.splay) {
            return Err(ImpossibleConfiguration {
                reason: format!(
                    "Jonction {} : aucun trou à {:.0}° sur \"{}\". Angles percés : {}.",
                    index + 1,
                    joint.splay,
                    upper.name,
                    format_splay_grid(&upper.mechanical.splay_grid)
                ),
            });
        }
    }

    let chain = models.iter().map(|m| ChainSpeaker::from_model(m)).collect();
    Ok(ResolvedChain {
        models,
        chain,
        recommended_splays,
    })
}

/// Un bumper est obligatoire (brief §11.6) : celui référencé par la grappe
/// doit être déclaré compatible avec l'enceinte et le compartiment, sinon
/// c'est une configuration impossible — jamais un repli silencieux vers un
/// pickup manuel ou l'absence de bumper, qui n'existent plus.
fn validate_bumper_compatible(
    bumper_model: &BumperModel,
    speaker_model: &SpeakerModel,
    compartment: Compartment,
) -> Result<(), ImpossibleConfiguration> {
    let compatible = bumper_model.compatible_speakers.iter().any(|c| {
        c.speaker_model_id == speaker_model.id
            && match compartment {
                Compartment::Flown => c.flown,
                Compartment::Stacked => c.stacked,
            }
    });
    if compatible {
        Ok(())
    } else {
        let compartment_label = match compartment {
            Compartment::Flown => "en vol",
            Compartment::Stacked => "en stack",
        };
        Err(ImpossibleConfiguration {
            reason: format!(
                "Le bumper \"{}\" n'est pas déclaré compatible avec l'enceinte \"{}\" {compartment_label}. Choisis un bumper compatible ou mets à jour sa compatibilité dans Équipement et enceinte.",
                bumper_model.name, speaker_model.name
            ),
        })
    }
}

/// La barre de déport n'est jamais choisie à la main : elle est entièrement
/// dérivée du bumper actif (celui qui déclare la compatibilité, brief §8).
fn bumper_bar_for<'a>(
    bumper_bars: &'a [BumperBarModel],
    bumper: &BumperModel,
) -> Option<&'a BumperBarModel> {
    bumper_bars.iter().find(|bumper_bar| {
        bumper_bar
            .compatible_bumpers
            .iter()
            .any(|c| c.bumper_model_id == bumper.id)
    })
}

/// Une assiette imposée n'est physiquement atteignable que si le solveur
/// trouve un moyen de la tenir : bumper seul, bumper + barre, ou en dernier
/// recours une tirette. Ce n'est jamais un résultat approximatif — brief §11.6.
#[derive(Clone, Debug)]
pub struct ImpossibleConfiguration {
    pub reason: String,
}

/// Efforts aux **deux pions** qui tiennent le bumper à l'enceinte de référence
/// en **stack** : le pion avant (charnière haute, `ht`) et le pion arrière (le
/// trou de cadre de calage, celui qui fixe l'assiette).
///
/// Le bumper est un corps rigide goupillé en deux points sur cette enceinte :
/// c'est exactement le problème de la paire ancrage/verrou, et il se résout de
/// la même façon — part directe partagée en deux, plus un couple perpendiculaire
/// à la ligne des deux pions, à raideurs égales. Deux pions dans un même corps,
/// ce sont 4 inconnues pour 3 équations : sans cette hypothèse, la répartition
/// n'est pas déterminée.
///
/// Rien de tout ceci n'existait en stack : le bumper y était purement
/// géométrique. Cette fonction n'altère donc aucune valeur déjà validée, elle
/// remplit un trou.
fn compute_stacked_bumper_pins(
    chain: &[ChainSpeaker],
    speakers: &[SpeakerInstance],
    pins_global: [Vec2; 2],
    g: f64,
    k_dyn: f64,
    share_per_flank: f64,
) -> (Vec2, Vec2, Vec2, Vec2) {
    // Perçage déclaré du bumper, avant puis arrière. Déjà en repère global : le
    // bumper posé au sol reste parallèle à celui-ci, il ne prend pas l'assiette
    // de l'enceinte.
    let [front, rear] = pins_global;

    let mut w_total = 0.0;
    let mut sum = Vec2::ZERO;
    for (speaker, instance) in chain.iter().zip(speakers) {
        let w = speaker.mass_kg * g * k_dyn;
        w_total += w;
        sum = sum + instance.cg * w;
    }
    let cm = sum * (1.0 / w_total);

    // Ce que le bumper reprend : le poids de tout ce qui est posé dessus.
    let load = Vec2::new(0.0, -w_total);
    let span = rear - front;
    let d = span.norm();
    let g_point = (front + rear) * 0.5;
    let m_g = (cm - g_point).cross(load);
    let perp = Vec2::new(-span.y, span.x) * (1.0 / d);
    let p = perp * (m_g / d);
    // `−load/2 ± P` est la réaction que les pions opposent au corps libre ; ce
    // que la quincaillerie **subit** en est l'opposé, comme partout ailleurs.
    // Sur un stack d'aplomb le couple s'annule et chaque pion voit `W/2` vers
    // le bas : la pile appuie sur le bumper, ce qui est le sens attendu.
    //
    // Par flanc : ces pions traversent les flancs de l'enceinte, ils sont
    // doublés — contrairement à la manille de levage, qui ne l'est pas.
    let f_front = (load * 0.5 - p) * share_per_flank;
    let f_rear = (load * 0.5 + p) * share_per_flank;
    (front, f_front, rear, f_rear)
}

/// Efforts transmis par le bumper à l'enceinte de référence (`speakers[0]`).
/// Corps libre = toutes les enceintes, puisque tout pend de ce point.
///
/// `pins_local` est le perçage **déclaré** du bumper (`BumperModel::pins`),
/// avant puis arrière, dans le repère de l'enceinte de référence. C'est là que
/// la barre est réellement boulonnée, donc là que la statique se résout : s'en
/// remettre à la quincaillerie de l'enceinte (charnière haute, trou d'ancrage)
/// donnait des bras de levier qui n'étaient pas ceux du montage, et qui
/// bougeaient avec le modèle d'enceinte monté dessous.
///
/// Deux pions goupillés dans un même corps rigide, ce sont 4 inconnues pour 3
/// équations : il faut une hypothèse de plus. C'est **la même** qu'en stack
/// (`compute_stacked_bumper_pins`) et qu'à toute autre paire de quincaillerie
/// (`split_over_pair`) : répartition élastique à raideurs égales — mêmes
/// goupilles, même perçage, même flanc.
///
/// Elle remplace un « bras arrière à deux forces » supposé **d'aplomb** du pion
/// arrière. Cette direction-là était fabriquée, pas mesurée : elle rendait
/// l'effort du pion arrière vertical quoi qu'il arrive, donc insensible au
/// déport réel de la manille, et colinéaire à la barre qui descend vers la paire
/// ancrage/verrou — d'où un couple quasi nul déversé dans l'enceinte de
/// référence (~60 N·m) là où toutes les jonctions voisines en passaient dix fois
/// plus. Le bumper se résolvait en vase clos ; il fait maintenant partie du même
/// ensemble que la manille, les enceintes et la tirette.
fn compute_bumper_loads(
    chain: &[ChainSpeaker],
    speakers: &[SpeakerInstance],
    pins_local: [Vec2; 2],
    settings: &Settings,
    tie: Option<TieForce>,
) -> BumperLoads {
    let (g, k_dyn, share_per_flank) = (
        settings.gravity,
        settings.dynamic_factor,
        settings.share_per_flank,
    );
    let b0 = speakers[0];
    let [front_pin_local, rear_pin_local] = pins_local;
    let pvg = b0.o + front_pin_local.rotate(b0.phi);
    let an = b0.o + rear_pin_local.rotate(b0.phi);
    // Barycentre des deux pions : c'est en ce point que le torseur se réduit,
    // donc en ce point que le moment extérieur doit être pris.
    let pins_g = (an + pvg) * 0.5;

    // Poids enceinte par enceinte (grappe hétérogène).
    let mut w_total = 0.0;
    let mut sum = Vec2::ZERO;
    for (speaker, instance) in chain.iter().zip(speakers) {
        let w = speaker.mass_kg * g * k_dyn;
        w_total += w;
        sum = sum + instance.cg * w;
    }
    let cm = sum * (1.0 / w_total);

    let mut rext = Vec2::new(0.0, -w_total);
    let mut mext = (cm - pins_g).cross(rext);
    if let Some(tie) = tie {
        let last = speakers[speakers.len() - 1];
        let q = last.o + tie.point_local.rotate(last.phi);
        rext = rext + tie.force;
        mext += (q - pins_g).cross(tie.force);
    }

    // Les deux pions équilibrent à eux seuls tout ce que subit le corps libre :
    // ils délivrent `−rext` et le moment `−mext`.
    let (f_ori, f_piv) = split_wrench_over_pair(an, pvg, -rext, -mext);

    // La barre qui descend du pion arrière est boulonnée au caisson par sa paire
    // ancrage/verrou — c'est elle qui verrouille la rotation de l'enceinte de
    // référence, exactement comme la barre d'une jonction. On lui applique donc
    // la même répartition : `f_ori` arrive à son extrémité haute (le pion
    // arrière), et les deux goupilles s'en partagent la résultante et le moment.
    let geo = &chain[0].geo;
    let pair_anchor_point = b0.o + geo.anchor_local.rotate(b0.phi);
    let pair_latch_point = b0.o + geo.latch_local.rotate(b0.phi);
    let (f_anchor_g, f_latch_g, m_g) =
        split_over_pair(pair_anchor_point, pair_latch_point, an, f_ori);

    let sg = -share_per_flank;
    BumperLoads {
        // Sens de la paire : `+share_per_flank`, comme dans une jonction — la
        // répartition part déjà de ce que la barre délivre aux goupilles, là où
        // `f_ori`/`f_piv` ci-dessous partent du corps libre et doivent être
        // retournés.
        pair_anchor_point,
        pair_latch_point,
        pair_anchor_force: f_anchor_g * share_per_flank,
        pair_latch_force: f_latch_g * share_per_flank,
        pair_moment_nm: m_g * share_per_flank / 1000.0,
        // Par flanc et retournés, comme partout ailleurs : ce que la
        // quincaillerie subit, pas ce qui agit sur le corps libre.
        orientation_force: f_ori * sg,
        orientation_point: an,
        pivot_force: f_piv * sg,
        pivot_point: pvg,
        // La manille, elle, n'est pas doublée : elle reprend la résultante
        // entière. `rext` est l'effort extérieur sur le corps libre (poids vers
        // le bas, plus la tirette) ; ce que la manille tire est son opposé.
        support_force: -rext,
    }
}

/// Calcule la géométrie, la statique de chaque jonction et la tension de tirette
/// éventuelle pour une grappe ou un stack (brief §4, §5).
///
/// La grappe est **hétérogène** : `speaker_models` est le catalogue complet, et
/// `Cluster::speaker_model_ids` désigne le modèle monté à chaque position, du
/// haut vers le bas. Chaque jonction consécutive doit être déclarée compatible
/// (`SpeakerModel::compatible_below`) pour ce compartiment, sinon la
/// configuration est impossible. Le splay recommandé éventuellement déclaré
/// pour la paire ressort dans `JointResult::acoustically_optimal` — un simple
/// signalement, jamais une erreur.
///
/// `bumper_model` est obligatoire : il doit être compatible avec l'enceinte de
/// référence (haut en vol, bas en stack) pour ce compartiment
/// (`validate_bumper_compatible`), sinon la configuration est impossible. Il
/// dérive le point d'accroche (vol) et fournit la silhouette à afficher. La
/// barre de déport n'est jamais choisie à la main : parmi `bumper_bars`, celle
/// qui déclare le bumper actif compatible (s'il y en a une) est utilisée
/// automatiquement.
///
/// En vol, le point d'accroche est **toujours** calculé (jamais saisi) :
/// centré par défaut, résolu par inversion si l'assiette est imposée. Si le
/// décalage nécessaire dépasse `max_direct_deport_mm`, une barre de déport
/// (SA303-BUMPER-BAR, composant d'équipement à part, non modélisé
/// géométriquement) est signalée nécessaire. Si même sa portée max ne suffit
/// pas, une tirette est automatiquement mise en place — sur le point de
/// splay 0 arrière-bas de l'enceinte du bas — pour tenir l'assiette exacte
/// malgré tout : ce n'est plus une case à cocher, c'est entièrement dérivé.
/// La direction de traction reste un choix utilisateur (`Cluster::tie_angle`,
/// `None` tant qu'il n'a pas choisi) : un câble ne peut que tirer, jamais
/// pousser, ce qui délimite une plage de directions valables
/// (`tie_valid_angle_range_deg`) — à l'utilisateur de choisir dedans selon où
/// se trouve un point d'ancrage réel, pas à l'algorithme de décider seul.
/// Reste à vérifier que la direction retenue est réellement dégagée : un vrai
/// test de collision (rayon contre la silhouette de chaque autre enceinte),
/// pas une simple comparaison de position relative. Quand la barre est
/// dépassée et qu'aucune direction valable n'est dégagée, la configuration
/// est physiquement impossible et remonte une erreur plutôt qu'un résultat
/// trompeur (brief §11.6).
pub fn compute_cluster(
    speaker_models: &[SpeakerModel],
    cluster: &Cluster,
    settings: &Settings,
    bumper_model: &BumperModel,
    bumper_bars: &[BumperBarModel],
) -> Result<ClusterResult, ImpossibleConfiguration> {
    let resolved = resolve_chain(speaker_models, cluster)?;
    let chain = resolved.chain.as_slice();
    let splays: Vec<f64> = cluster.joints.iter().map(|j| j.splay).collect();
    let compartment = cluster.compartment;

    // Le bumper est porté par l'enceinte de référence : celle du haut en vol,
    // celle du bas en stack — c'est donc sa compatibilité à elle qui compte,
    // pas celle d'une position quelconque de la chaîne.
    let reference_index = match compartment {
        Compartment::Flown => 0,
        Compartment::Stacked => resolved.models.len() - 1,
    };
    let reference_model = resolved.models[reference_index];
    validate_bumper_compatible(bumper_model, reference_model, compartment)?;
    let active_bumper_bar = bumper_bar_for(bumper_bars, bumper_model);

    let phi_initial = match compartment {
        Compartment::Stacked => {
            let bottom_deg = cluster.imposed_tilt.unwrap_or(0.0);
            phi_initial_stack(bottom_deg, &splays)
        }
        Compartment::Flown => match cluster.imposed_tilt {
            Some(deg) => deg.to_radians(),
            None => {
                let pickup_height = bumper_pickup_height(reference_model, bumper_model);
                phi_initial_free_hang(chain, &splays, Vec2::new(0.0, pickup_height))
            }
        },
    };

    // Référence de comparaison, toujours calculée en vol (jamais en stack, où
    // la notion n'a pas de sens) : l'angle qu'adopterait la grappe accrochée
    // au centre du bumper, sans assiette imposée. Reste distincte de
    // `phi_initial` même quand une assiette est imposée — les deux doivent
    // s'afficher côte à côte, jamais l'une remplacer l'autre (brief,
    // correction utilisateur).
    let phi_free_hang = match compartment {
        Compartment::Stacked => None,
        Compartment::Flown => {
            let pickup_height = bumper_pickup_height(reference_model, bumper_model);
            Some(phi_initial_free_hang(
                chain,
                &splays,
                Vec2::new(0.0, pickup_height),
            ))
        }
    };

    let speakers = build_cluster(chain, &splays, phi_initial);

    // Poids et CG de l'ensemble, une fois pour toutes : la tirette ne
    // s'intéresse qu'à la résultante, pas à la répartition enceinte par
    // enceinte (qui, elle, compte pour chaque jonction).
    let total_mass_kg: f64 = chain.iter().map(|c| c.mass_kg).sum();
    let total_weight_n = total_mass_kg * settings.gravity * settings.dynamic_factor;
    let cluster_cg = weighted_cg(chain, &speakers);

    // Point d'accroche final, et tirette auto-décidée par le solveur quand le
    // bumper (+ sa barre) ne suffit plus à atteindre seul l'assiette voulue.
    let (pickup, bumper_bar_exceeded, auto_tie, tie_angle_range_deg): (
        Vec2,
        bool,
        Option<TieForce>,
        Option<(f64, f64)>,
    ) = match compartment {
        Compartment::Stacked => (Vec2::ZERO, false, None, None),
        Compartment::Flown => {
            let pickup_height = bumper_pickup_height(reference_model, bumper_model);
            match cluster.imposed_tilt {
                None => (Vec2::new(0.0, pickup_height), false, None, None),
                Some(_) => {
                    let raw_x =
                        solve_pickup_x_for_imposed_tilt(chain, &splays, phi_initial, pickup_height);
                    // Sans barre compatible, la zone de fixation directe est la
                    // seule portée disponible : au-delà, il faut déjà une tirette.
                    let bumper_bar_max = active_bumper_bar
                        .map(|bumper_bar| bumper_bar.max_deport_mm)
                        .unwrap_or(bumper_model.max_direct_deport_mm);
                    if raw_x.abs() <= bumper_bar_max {
                        (Vec2::new(raw_x, pickup_height), false, None, None)
                    } else {
                        // Point 0° arrière-bas de l'enceinte
                        let tie_point = chain[chain.len() - 1].geo.crown(0.0);
                        let capped_x = bumper_bar_max * raw_x.signum();
                        let capped_pickup = Vec2::new(capped_x, pickup_height);
                        let range = tie_valid_angle_range_deg(
                            &speakers,
                            total_weight_n,
                            cluster_cg,
                            capped_pickup,
                            tie_point,
                        );
                        // La direction de traction dépend du terrain (où se
                        // trouve un point d'ancrage réel) : ce n'est pas à
                        // l'algorithme de la choisir seul. Tant que
                        // l'utilisateur n'a pas encore choisi, on suggère la
                        // plus proche de 180° (vers l'arrière, la direction la
                        // plus courante en pratique) dans la plage valable —
                        // jamais imposée.
                        let tie_angle = cluster
                            .tie_angle
                            .unwrap_or_else(|| tie_default_angle_deg(range.0, range.1));
                        let tension = tie_tension(
                            &speakers,
                            total_weight_n,
                            cluster_cg,
                            capped_pickup,
                            tie_point,
                            tie_angle,
                        );
                        // Un câble ne peut que tirer : une tension négative
                        // signifie que cet angle précis demanderait de
                        // pousser — physiquement impossible dans cette
                        // direction, quelle que soit la collision.
                        if tension < 0.0 {
                            return Err(ImpossibleConfiguration {
                                reason: format!(
                                    "Assiette impossible à tenir avec une tirette à {tie_angle:.0}° : il faudrait pousser au lieu de tirer. Choisis un angle entre {:.0}° et {:.0}°.",
                                    range.0, range.1
                                ),
                            });
                        }
                        let tie_dir = dir_from_angle(tie_angle);
                        let last = speakers[speakers.len() - 1];
                        let tie_point_global = last.o + tie_point.rotate(last.phi);
                        // Reste à vérifier que cette direction est
                        // réellement dégagée : un vrai test de collision
                        // (rayon contre la silhouette de chaque autre
                        // enceinte), pas une simple comparaison de position
                        // relative. Seules les AUTRES enceintes comptent :
                        // celle du bas ne peut pas se bloquer elle-même.
                        let blocked = speakers[..speakers.len() - 1].iter().any(|sp| {
                            let corners = sp.outline.map(|c| sp.o + c.rotate(sp.phi));
                            ray_hits_polygon(tie_point_global, tie_dir, &corners)
                        });
                        if blocked {
                            return Err(ImpossibleConfiguration {
                                reason: format!(
                                    "Assiette impossible à tenir : le déport nécessaire ({raw_x:.0} mm) dépasse la portée de la barre ({bumper_bar_max:.0} mm) et la tirette à {tie_angle:.0}° traverserait une autre enceinte de la grappe. Choisis un autre angle entre {:.0}° et {:.0}°, ou réduis l'assiette imposée.",
                                    range.0, range.1
                                ),
                            });
                        }
                        let force = TieForce {
                            point_local: tie_point,
                            force: tie_dir * tension,
                        };
                        (capped_pickup, true, Some(force), Some(range))
                    }
                }
            }
        }
    };

    // Pas de tirette manuelle au sens "case à cocher" (brief, correction
    // utilisateur) : la tirette n'existe que si le solveur l'a activée
    // ci-dessus. Sa direction, elle, reste un choix utilisateur dans la plage
    // affichée (`tie_angle_range_deg`) — l'algorithme délimite, ne décide pas.
    let tie_force = auto_tie;
    let tension = tie_force.map(|t| t.force.norm()).unwrap_or(0.0);

    let joints = (0..splays.len())
        .map(|i| {
            let input = JointInput {
                chain,
                speakers: &speakers,
                splays_deg: &splays,
                joint_index: i,
                compartment,
                g: settings.gravity,
                k_dyn: settings.dynamic_factor,
                share_per_flank: settings.share_per_flank,
                tie: tie_force,
                recommended_splay: resolved.recommended_splays[i],
            };
            compute_joint(&input)
        })
        // Une jonction sans solution n'est pas un résultat dégradé : c'est une
        // configuration impossible, au même titre qu'un splay sans trou percé.
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| ImpossibleConfiguration {
            reason: format!("Jonction {} : {}", e.joint_index + 1, e.reason),
        })?;

    let pickup_global =
        (compartment == Compartment::Flown).then(|| speakers[0].o + pickup.rotate(speakers[0].phi));

    let tie_point_global = tie_force.map(|t| {
        let last = speakers[speakers.len() - 1];
        last.o + t.point_local.rotate(last.phi)
    });
    let tie_direction_global = tie_force.map(|t| t.force.normalize());
    let tie_direction_angle_deg = tie_direction_global.map(angle_of);

    let bumper_view = match compartment {
        Compartment::Flown => {
            let attach = speakers[0];
            let outline_local = bumper_outline_top(reference_model, bumper_model);
            let outline_global = outline_local.map(|p| attach.o + p.rotate(attach.phi));
            let threshold = bumper_model.max_direct_deport_mm;
            let deport = if pickup.x.abs() > threshold {
                pickup.x - threshold * pickup.x.signum()
            } else {
                0.0
            };
            let bumper_bar_start_global = (deport != 0.0).then(|| {
                let edge_x = threshold * pickup.x.signum();
                attach.o + Vec2::new(edge_x, pickup.y).rotate(attach.phi)
            });
            // Perçage déclaré du bumper, exprimé dans le repère de l'enceinte de
            // référence : en vol le bumper est rigidement fixé à elle.
            let pins_local = bumper_pin_points(&outline_local, bumper_model);
            let loads =
                compute_bumper_loads(chain, &speakers, pins_local, settings, tie_force);
            let of_local = loads.orientation_force.rotate_transpose(attach.phi);
            let pf_local = loads.pivot_force.rotate_transpose(attach.phi);
            let pickup_g = attach.o + pickup.rotate(attach.phi);
            BumperView {
                outline_global,
                pickup_global: Some(pickup_g),
                bumper_bar_start_global,
                pickup_offset_mm: Some(pickup.x),
                bar_deport_mm: Some(deport),
                bumper_bar_exceeded,
                tie_angle_range_deg: tie_angle_range_deg.map(|(lo, hi)| [lo, hi]),
                orientation_force_n: Some(of_local.norm()),
                orientation_angle_deg: Some(angle_of(of_local)),
                pivot_force_n: Some(pf_local.norm()),
                pivot_angle_deg: Some(angle_of(pf_local)),
                orientation_point_global: Some(loads.orientation_point),
                pivot_point_global: Some(loads.pivot_point),
                pair_anchor_hole_global: Some(loads.pair_anchor_point),
                pair_latch_hole_global: Some(loads.pair_latch_point),
                f_pair_anchor_global: Some(loads.pair_anchor_force),
                f_pair_latch_global: Some(loads.pair_latch_force),
                f_pair_anchor_n: Some(loads.pair_anchor_force.norm()),
                f_pair_anchor_angle_deg: Some(angle_of(
                    loads.pair_anchor_force.rotate_transpose(attach.phi),
                )),
                f_pair_latch_n: Some(loads.pair_latch_force.norm()),
                f_pair_latch_angle_deg: Some(angle_of(
                    loads.pair_latch_force.rotate_transpose(attach.phi),
                )),
                pair_moment_nm: Some(loads.pair_moment_nm),
                orientation_force_global: Some(loads.orientation_force),
                pivot_force_global: Some(loads.pivot_force),
                support_force_n: loads.support_force.norm(),
                support_force_global: loads.support_force,
                support_point_global: pickup_g,
                support_angle_deg: angle_of(loads.support_force),
                pin_pair_moment_nm: pin_pair_moment_nm(
                    loads.orientation_point,
                    loads.orientation_force,
                    loads.pivot_point,
                    loads.pivot_force,
                ),
                pin_span_mm: (loads.orientation_point - loads.pivot_point).norm(),
            }
        }
        Compartment::Stacked => {
            // Le bumper reste toujours parallèle au sol en stack : c'est
            // l'enceinte de référence qui prend l'angle de calage, pas lui
            // (contrairement au vol, où il est rigidement fixé à l'enceinte).
            // Le point de contact est précisément le coin avant-bas de
            // l'enceinte (repère enceinte : indice 3 de `speaker_outline`,
            // avant = x négatif) une fois tournée — jamais son centre-bas non
            // tourné, ni le coin le plus bas au sens large : c'est ce coin-là,
            // et seulement lui, qui pose sur le bumper quand l'enceinte est
            // calée nez vers le bas, le reste se soulevant à l'arrière.
            let attach = speakers[speakers.len() - 1];
            let front_bottom_local = chain[reference_index].outline[3];
            let front_bottom_global = attach.o + front_bottom_local.rotate(attach.phi);
            let outline_global = [
                front_bottom_global,
                front_bottom_global + Vec2::new(bumper_model.depth, 0.0),
                front_bottom_global + Vec2::new(bumper_model.depth, -bumper_model.height),
                front_bottom_global + Vec2::new(0.0, -bumper_model.height),
            ];
            // En stack, le bumper porte au lieu de suspendre. La réaction du
            // sol remonte la totalité du poids dynamisé, appliquée au milieu de
            // la face d'appui — le bumper repose à plat, donc la résultante n'a
            // pas de raison d'être ailleurs.
            let support = Vec2::new(0.0, total_weight_n);
            let support_point = (outline_global[2] + outline_global[3]) * 0.5;
            // Et les deux pions qui le tiennent à l'enceinte du bas.
            let pins_global = bumper_pin_points(&outline_global, bumper_model);
            let (front_pt, front_f, rear_pt, rear_f) = compute_stacked_bumper_pins(
                chain,
                &speakers,
                pins_global,
                settings.gravity,
                settings.dynamic_factor,
                settings.share_per_flank,
            );
            let front_local = front_f.rotate_transpose(attach.phi);
            let rear_local = rear_f.rotate_transpose(attach.phi);
            BumperView {
                outline_global,
                pickup_global: None,
                bumper_bar_start_global: None,
                pickup_offset_mm: None,
                bar_deport_mm: None,
                bumper_bar_exceeded: false,
                tie_angle_range_deg: None,
                // Le pion arrière (trou de cadre) joue le rôle « orientation »,
                // le pion avant (charnière) celui du pivot : mêmes noms qu'en
                // vol, pour que le front n'ait pas deux schémas à gérer.
                orientation_force_n: Some(rear_local.norm()),
                orientation_angle_deg: Some(angle_of(rear_local)),
                pivot_force_n: Some(front_local.norm()),
                pivot_angle_deg: Some(angle_of(front_local)),
                orientation_point_global: Some(rear_pt),
                pivot_point_global: Some(front_pt),
                // En stack le bumper est goupillé sans barre : pas de paire à
                // encastrer, donc rien à rendre ici.
                pair_anchor_hole_global: None,
                pair_latch_hole_global: None,
                f_pair_anchor_global: None,
                f_pair_latch_global: None,
                f_pair_anchor_n: None,
                f_pair_anchor_angle_deg: None,
                f_pair_latch_n: None,
                f_pair_latch_angle_deg: None,
                pair_moment_nm: None,
                orientation_force_global: Some(rear_f),
                pivot_force_global: Some(front_f),
                support_force_n: support.norm(),
                support_force_global: support,
                support_point_global: support_point,
                support_angle_deg: angle_of(support),
                pin_pair_moment_nm: pin_pair_moment_nm(rear_pt, rear_f, front_pt, front_f),
                pin_span_mm: (rear_pt - front_pt).norm(),
            }
        }
    };

    let elevation = compute_elevation(
        cluster.bumper_height,
        &speakers,
        &bumper_view,
        pickup_global,
    );

    Ok(ClusterResult {
        speakers,
        phi_initial,
        phi_free_hang,
        tie_tension_n: tension,
        joints,
        cg: cluster_cg,
        total_mass_kg,
        speaker_names: resolved.models.iter().map(|m| m.name.clone()).collect(),
        pickup_global,
        tie_point_global,
        tie_direction_global,
        tie_direction_angle_deg,
        elevation,
        bumper_view,
    })
}

/// Situe la grappe dans l'espace. Le repère de calcul a son origine sur
/// l'enceinte de référence, donc ses `y` sont relatifs : c'est la hauteur de
/// bumper déclarée qui lui donne une altitude, en calant le **dessous** du
/// bumper — le point qui touche le sol en stack — sur la valeur saisie.
///
/// Rien ici n'entre dans la statique : une grappe pèse le même poids à 2 m
/// qu'à 12 m. C'est de la mise en situation, calculée ici pour que l'écran
/// n'ait jamais à additionner un décalage à une ordonnée (brief §1).
fn compute_elevation(
    bumper_height: f64,
    speakers: &[SpeakerInstance],
    bumper_view: &BumperView,
    pickup_global: Option<Vec2>,
) -> Elevation {
    let bumper_ys = bumper_view.outline_global.iter().map(|p| p.y);
    let bumper_bottom_y = bumper_ys.fold(f64::INFINITY, f64::min);
    let offset_mm = bumper_height - bumper_bottom_y;

    // Silhouettes réellement tracées : les coins des enceintes une fois
    // tournées, plus ceux du bumper. C'est l'encombrement vu à l'écran.
    let mut lowest = f64::INFINITY;
    let mut highest = f64::NEG_INFINITY;
    let mut speaker_bottom_mm = Vec::with_capacity(speakers.len());
    for speaker in speakers {
        let mut bottom = f64::INFINITY;
        for corner in speaker.outline {
            let y = (speaker.o + corner.rotate(speaker.phi)).y;
            bottom = bottom.min(y);
            highest = highest.max(y);
        }
        lowest = lowest.min(bottom);
        speaker_bottom_mm.push(bottom + offset_mm);
    }
    for corner in bumper_view.outline_global {
        lowest = lowest.min(corner.y);
        highest = highest.max(corner.y);
    }

    Elevation {
        offset_mm,
        bumper_bottom_mm: bumper_height,
        lowest_point_mm: lowest + offset_mm,
        highest_point_mm: highest + offset_mm,
        pickup_mm: pickup_global.map(|p| p.y + offset_mm),
        speaker_bottom_mm,
    }
}

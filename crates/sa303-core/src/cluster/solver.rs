//! Assemblage d'une grappe ou d'un stack entier et décision du solveur
//! (brief §4, §5) : dérive le point d'accroche depuis le bumper obligatoire,
//! décide si une barre puis un pull-back sont nécessaires, calcule chaque
//! jonction, et rend le tout exploitable par le front (`ClusterResult`) sans
//! qu'il ait jamais à recalculer quoi que ce soit.

use super::joint::{compute_joint, JointInput, JointResult, MIN_BIELLE_LEVER_MM};
use super::kinematics::{
    build_cluster, phi_initial_free_hang, phi_initial_stack, solve_pickup_x_for_imposed_tilt,
    weighted_cg, ChainSpeaker, SpeakerInstance,
};
use super::model::{Cluster, Compartment};
use super::pair::split_over_pair;
use super::rigging::{
    pick_pair, pick_single, rigging_options, single_tilt_range, BarMountView, Candidate,
    GroupKind, LinkForceView, RiggingOptions, RiggingPointView, RiggingView, TILT_TOLERANCE_DEG,
};
use super::model::RiggingSupport;
use crate::bumper::{
    bar_outline_local, bumper_outline_top, bumper_pickup_height, bumper_pin_points, BumperBarModel,
    BumperModel,
    BumperRearBar,
};
use crate::settings::Settings;
use crate::speaker::{SpeakerModel, SplayRange};
use crate::pull_back::{
    clamp_to_pull_back_range, pull_back_usable_range_deg, pull_back_window_deg, pull_back_default_angle_deg,
    PULL_BACK_VERTICAL_DEG,
    pull_back_tension, pull_back_valid_angle_range_deg, PullBackForce,
};
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
    pub pull_back_tension_n: f64,
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
    /// Point d'accroche du pull-back sur l'enceinte du bas, repère global.
    pub pull_back_point_global: Option<Vec2>,
    /// Direction de traction du pull-back, repère global (convention §2).
    pub pull_back_direction_global: Option<Vec2>,
    /// Même direction, en degrés (convention §2 : 180° = vers le haut ; un
    /// pull-back reste dans 180° ± `Settings::pull_back_tolerance_deg`) — pour affichage : le rigger doit connaître
    /// l'angle réel auquel ancrer le pull-back. Choisi par l'utilisateur, ou
    /// suggérée par le solveur tant qu'il n'a pas encore choisi.
    pub pull_back_direction_angle_deg: Option<f64>,
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
    /// Au-delà de la portée de la barre (`BumperBarModel::max_deport_mm`), elle ne suffit plus : un
    /// pull-back est obligatoire et mis en place automatiquement (voir
    /// `pull_back_tension_n`, `pull_back_point_global`). Un pull-back manuel
    /// (`Cluster::pull_back_enabled`) laisse ce drapeau à `false`.
    pub bumper_bar_exceeded: bool,
    /// Plage de directions de traction physiquement valables pour le pull-back
    /// (degrés, convention §2), présente dès qu'un pull-back est en place,
    /// imposé ou manuel. Le
    /// point d'ancrage réel dépend du terrain : à l'utilisateur de choisir
    /// dedans, pas à l'algorithme.
    pub pull_back_angle_range_deg: Option<[f64; 2]>,
    /// Pull-back manuel seulement : tensions saisissables, N (même base que
    /// les autres efforts, poids × k_dyn). Elles correspondent aux positions
    /// d'accroche atteignables sur le bumper et sa barre ; l'accroche qui
    /// donne la tension demandée est dans `pickup_offset_mm`.
    pub pull_back_tension_range_n: Option<[f64; 2]>,
    /// Part du poids dynamisé reprise par le pull-back (composante verticale
    /// de sa tension / poids), 0 sans pull-back. La manille porte le reste.
    pub pull_back_load_share: f64,
    /// Efforts transmis par le bumper à l'enceinte de référence, repère de
    /// cette enceinte : ce que chacun de ses **deux pions** encaisse.
    /// `orientation` est le pion arrière (celui qui alimente la barre),
    /// `pivot` le pion avant (celui de la bielle) — mêmes rôles et mêmes noms
    /// qu'à une jonction. Vol uniquement. Par flanc, comme les efforts de
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
    /// toute la grappe, pull-back compris.
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
    /// Vol, bumper aux trous déclarés : trous retenus, charge de chacun,
    /// montage de barre et assiette réellement obtenue. `None` sinon.
    pub rigging: Option<RiggingView>,
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
/// recours un pull-back. Ce n'est jamais un résultat approximatif — brief §11.6.
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
/// Le bumper est placé par ses **deux liaisons réelles** — la bielle avant et
/// la barre arrière — et non par le perçage coté depuis ses bords
/// (`BumperModel::pins`). Les deux placements décrivent les mêmes trous, mais
/// seul celui-ci ferme : les cotes de bords sont arrondies au mm, et la
/// fermeture se joue au centième.
///
/// La liaison est **celle d'une jonction**, pas une paire de goupilles libres :
///
/// * à l'avant, une bielle bi-goupillée relie le trou avant du bumper à la
///   charnière haute de l'enceinte (`geo.ht`). Élément à deux forces : sa
///   direction est imposée par ses deux goupilles, donc une seule inconnue
///   scalaire, exactement comme la bielle avant de `compute_joint` ;
/// * à l'arrière, la barre du bumper descend du pion arrière et se boulonne au
///   caisson par sa paire ancrage/verrou.
///
/// Ce sont 3 inconnues pour 3 équations : la liaison est déterminée, sans
/// hypothèse de raideur à inventer.
///
/// Elle remplace une répartition élastique 50/50 sur les deux pions
/// (`split_wrench_over_pair`). Cette hypothèse-là laissait le pion avant
/// reprendre la moitié de la verticale, et envoyait le couple perpendiculaire à
/// l'entraxe des pions — c'est-à-dire, l'entraxe étant l'axe du caisson,
/// **quasi le long de l'axe de la barre**. L'effort du pion arrière en sortait
/// presque purement axial : la barre travaillait en compression au lieu de la
/// flexion. Vérifié par `the_front_bumper_pin_pulls_along_its_bielle` et
/// `the_reference_bar_takes_more_than_the_next_one`.
///
/// Le point d'application arrière est le **trou haut de la barre**
/// (`BumperRearBar`), construit depuis la paire ancrage/verrou du caisson — pas
/// le pion déduit de la silhouette du bumper par `bumper_pin_points`. Ce
/// dernier repose sur `height_from_bottom_mm`, une cote approchée qui place le
/// point à quelques millimètres près et, surtout, ne connaît pas la longueur de
/// la barre réellement montée : c'est pourtant elle qui fixe le bras, donc ce
/// que la paire encaisse. Vérifié par `the_bumper_bar_sets_the_pair_lever`.
fn compute_bumper_loads(
    chain: &[ChainSpeaker],
    speakers: &[SpeakerInstance],
    pivot_bar_usable_length_mm: f64,
    rear_bar: &BumperRearBar,
    settings: &Settings,
    pull_back: Option<PullBackForce>,
) -> Result<BumperLoads, ImpossibleConfiguration> {
    let (g, k_dyn, share_per_flank) = (
        settings.gravity,
        settings.dynamic_factor,
        settings.share_per_flank,
    );
    let b0 = speakers[0];
    let geo = &chain[0].geo;
    // Goupille basse de la bielle avant : la charnière haute du caisson.
    let bielle_low = b0.o + geo.ht.rotate(b0.phi);
    // Goupille haute : au bout de la bielle, d'aplomb **dans le repère du
    // caisson** — le bumper est rigidement fixé à lui en vol, donc la bielle
    // garde la même orientation relative quelle que soit l'assiette. C'est la
    // bielle qui place ce point, pas `height_from_bottom_mm` : cette cote-là est
    // arrondie et laissait le pion 6,3 mm au-dessus du trou haut de la barre,
    // alors que les deux sont percés dans la même pièce.
    let pvg = b0.o + (geo.ht + Vec2::new(0.0, pivot_bar_usable_length_mm)).rotate(b0.phi);

    // La paire ancrage/verrou par laquelle la barre est boulonnée, et le repère
    // de barre qu'elle définit : abscisse du verrou vers l'ancrage, latéral vers
    // l'avant du caisson — exactement la construction de `compute_joint`.
    let pair_anchor_point = b0.o + geo.anchor_local.rotate(b0.phi);
    let pair_latch_point = b0.o + geo.latch_local.rotate(b0.phi);
    let e_axis = (pair_anchor_point - pair_latch_point).normalize();
    let e_front = Vec2::new(-e_axis.y, e_axis.x);
    // Trou haut de la barre : c'est lui qui reçoit l'effort, et son éloignement
    // de la paire est le bras qui décide de ce qu'elle encaisse.
    let an = pair_anchor_point
        + e_axis * rear_bar.top_hole_along_mm
        + e_front * rear_bar.top_hole_lateral_mm;
    // Moment extérieur pris au pion arrière : c'est lui qui porte l'inconnue
    // vectorielle, donc c'est là qu'elle disparaît de l'équation de moment.
    let pins_g = an;

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
    if let Some(pull_back) = pull_back {
        let last = speakers[speakers.len() - 1];
        let q = last.o + pull_back.point_local.rotate(last.phi);
        rext = rext + pull_back.force;
        mext += (q - pins_g).cross(pull_back.force);
    }

    // Bielle avant, élément à deux forces : direction imposée par ses deux
    // goupilles, donc une seule inconnue scalaire.
    let u = (pvg - bielle_low).normalize();
    let bielle_lever = (pvg - an).cross(u);
    // Même garde qu'en jonction : si la ligne d'action de la bielle passe par le
    // pion arrière, `lambda` part à l'infini et des NaN traverseraient tout le
    // calcul sans qu'aucun seuil ne les arrête (les comparaisons sur NaN sont
    // fausses). Ce n'est atteignable que sur un perçage aberrant.
    if bielle_lever.abs() < MIN_BIELLE_LEVER_MM {
        return Err(ImpossibleConfiguration {
            reason: format!(
                "Bumper : bras de bielle {bielle_lever:.4} mm — la ligne d'action de la \
                 bielle avant passe par le pion arrière, la liaison n'a pas de solution."
            ),
        });
    }
    let f_piv = u * (-mext / bielle_lever);
    // Le pion arrière reprend tout le reste : c'est par lui que la barre du
    // bumper, encastrée sur le caisson, passe son effort.
    let f_ori = -rext - f_piv;

    // La barre qui descend du pion arrière est boulonnée au caisson par sa paire
    // ancrage/verrou — c'est elle qui verrouille la rotation de l'enceinte de
    // référence, exactement comme la barre d'une jonction. On lui applique donc
    // la même répartition : `f_ori` arrive à son extrémité haute (le pion
    // arrière), et les deux goupilles s'en partagent la résultante et le moment.
    let (f_anchor_g, f_latch_g, m_g) =
        split_over_pair(pair_anchor_point, pair_latch_point, an, f_ori);

    let sg = -share_per_flank;
    Ok(BumperLoads {
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
        // le bas, plus le pull-back) ; ce que la manille tire est son opposé.
        support_force: -rext,
    })
}

/// Accroche et pull-back retenus en vol (valeurs neutres en stack).
struct FlownSupport {
    pickup: Vec2,
    /// Le bumper et sa barre ne suffisent pas : le pull-back est obligatoire.
    pull_back_forced: bool,
    pull_back: Option<PullBackForce>,
    angle_range: Option<(f64, f64)>,
    /// Pull-back manuel seulement : tensions saisissables, N.
    tension_range: Option<(f64, f64)>,
    /// Trous retenus, quand le bumper les déclare.
    rig: Option<RigSelection>,
}

/// Trous retenus parmi ceux du bumper et de sa barre, avec la charge de
/// chacun quand elle est connue d'avance (2 points). À un point, la manille
/// reprend l'effort de support du bumper.
struct RigSelection {
    group: GroupKind,
    points: Vec<(Candidate, Option<f64>)>,
}

/// Ce que le solveur fait des trous déclarés, décidé avant d'assembler la
/// grappe : c'est ce choix qui fixe `φ_initial`.
enum RigPlan {
    /// Un point, l'assiette découle du trou. `forced_pull_back` : même le
    /// meilleur trou n'approche pas l'assiette visée, un pull-back la tient.
    Single { group: GroupKind, candidate: Candidate, forced_pull_back: bool },
    /// Un point plus un pull-back manuel : le trou sera choisi sur la tension.
    ManualPullBack,
    /// Deux points : l'assiette est libre, les trous se choisissent sur la
    /// répartition.
    Pair,
}

impl FlownSupport {
    fn centered(pickup: Vec2) -> Self {
        Self {
            pickup,
            pull_back_forced: false,
            pull_back: None,
            angle_range: None,
            tension_range: None,
            rig: None,
        }
    }
}

/// Nombre de pas du balayage de l'accroche en pull-back manuel.
const PICKUP_SWEEP_STEPS: usize = 400;

/// Positions d'accroche (mm depuis le centre du bumper) où un pull-back
/// vertical est tendu, en mode manuel. À `raw_x`, l'accroche tient seule
/// l'assiette : tension nulle. En s'éloignant de `raw_x`, la tension croît
/// d'un seul côté ; on part de ce côté jusqu'au bout de barre, et on s'arrête
/// avant tout point où la tension cesse d'être positive et finie (câble à
/// l'aplomb de l'accroche). `None` si aucun côté ne tend le câble.
fn manual_pickup_range_mm(
    raw_x: f64,
    bar_max: f64,
    tension_at: impl Fn(f64) -> f64,
) -> Option<(f64, f64)> {
    let side = [bar_max, -bar_max]
        .into_iter()
        .filter(|&e| (e - raw_x).abs() > 1e-6)
        .map(|e| {
            let probe = raw_x + (e - raw_x) * 1e-3;
            (e, tension_at(probe))
        })
        .filter(|&(_, t)| t.is_finite() && t > 0.0)
        .map(|(e, _)| e)
        .next()?;
    // La plage commence au premier pas, pas à `raw_x` : là, le câble est
    // exactement détendu et sa direction n'est plus définie.
    let mut first_ok = None;
    let mut last_ok = None;
    for i in 1..=PICKUP_SWEEP_STEPS {
        let x = raw_x + (side - raw_x) * (i as f64) / (PICKUP_SWEEP_STEPS as f64);
        let t = tension_at(x);
        if !(t.is_finite() && t > 0.0) {
            break;
        }
        first_ok.get_or_insert(x);
        last_ok = Some(x);
    }
    let (a, b) = (first_ok?, last_ok?);
    ((b - a).abs() > 1e-6).then(|| (a.min(b), a.max(b)))
}

/// Accroche où le pull-back a exactement la tension `target`, par dichotomie
/// sur `range` : la tension y varie de façon monotone (le balayage de
/// `manual_pickup_range_mm` s'est arrêté avant toute singularité).
fn pickup_for_tension_mm(range: (f64, f64), target: f64, tension_at: impl Fn(f64) -> f64) -> f64 {
    let (mut lo, mut hi) = if tension_at(range.0) <= tension_at(range.1) {
        (range.0, range.1)
    } else {
        (range.1, range.0)
    };
    for _ in 0..80 {
        let mid = 0.5 * (lo + hi);
        if tension_at(mid) < target {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    0.5 * (lo + hi)
}

/// Direction, tension et test de collision du pull-back, une fois l'accroche
/// fixée. Commun au pull-back imposé et manuel, avec ou sans trous déclarés.
fn resolve_pull_back(
    speakers: &[SpeakerInstance],
    total_weight_n: f64,
    cluster_cg: Vec2,
    pull_back_point: Vec2,
    pickup: Vec2,
    requested_angle: Option<f64>,
    settings: &Settings,
) -> Result<(PullBackForce, (f64, f64)), ImpossibleConfiguration> {
    let tension_at = |pickup: Vec2, angle: f64| {
        pull_back_tension(speakers, total_weight_n, cluster_cg, pickup, pull_back_point, angle)
    };
    let static_range = pull_back_valid_angle_range_deg(
        speakers,
        total_weight_n,
        cluster_cg,
        pickup,
        pull_back_point,
    );
    // Le pull-back est un second point de levage : il
    // tire verticalement vers le haut, 180° ± tolérance
    // (convention §2, pratique Meyer Sound). Seule la
    // partie de cette fenêtre où le câble tire (tension
    // positive) est utilisable. Si elle est vide, il
    // faudrait tirer vers le bas : ce n'est pas un
    // pull-back, la configuration est refusée.
    let tol = settings.pull_back_tolerance_deg;
    let Some(range) = pull_back_usable_range_deg(static_range, tol) else {
        return Err(ImpossibleConfiguration {
            reason: format!(
                "Assiette impossible à tenir : depuis l'accroche à {:.0} mm du centre du bumper, un pull-back vertical (180° ± {tol:.0}°) ne peut pas tenir cette assiette ; il faudrait tirer le bas de la grappe vers le bas. Réduis l'assiette imposée.",
                pickup.x
            ),
        });
    };
    // L'angle exact reste un choix utilisateur, ramené
    // dans la plage : le champ de saisie le borne déjà,
    // mais une grappe enregistrée peut en sortir quand
    // l'assiette change. Tant qu'il n'a pas choisi, on
    // suggère la verticale (ou la borne la plus proche).
    // Seule la collision reste une erreur.
    let angle = requested_angle
        .map(|a| clamp_to_pull_back_range(a, range))
        .unwrap_or_else(|| pull_back_default_angle_deg(range));
    let tension = tension_at(pickup, angle);
    // Garde-fou : dans la plage, le câble tire toujours.
    debug_assert!(tension > 0.0, "pull-back à {angle}° : tension {tension}");
    let dir = dir_from_angle(angle);
    let last = speakers[speakers.len() - 1];
    let point_global = last.o + pull_back_point.rotate(last.phi);
    // Un vrai test de collision (rayon contre la silhouette
    // de chaque autre enceinte). Seules les AUTRES
    // enceintes comptent : celle du bas ne peut pas se
    // bloquer elle-même.
    let blocked = speakers[..speakers.len() - 1].iter().any(|sp| {
        let corners = sp.outline.map(|c| sp.o + c.rotate(sp.phi));
        ray_hits_polygon(point_global, dir, &corners)
    });
    if blocked {
        return Err(ImpossibleConfiguration {
            reason: format!(
                "Assiette impossible à tenir : le pull-back à {angle:.0}° traverserait une autre enceinte de la grappe. Choisis un autre angle entre {:.0}° et {:.0}°, ou réduis l'assiette imposée.",
                range.0, range.1
            ),
        });
    }
    Ok((
        PullBackForce {
            point_local: pull_back_point,
            force: dir * tension,
        },
        range,
    ))
}

/// Décide ce que le solveur fera des trous déclarés — avant d'assembler la
/// grappe, puisque c'est ce choix qui fixe `φ_initial` à un point.
fn plan_rigging(
    options: &RiggingOptions,
    cluster: &Cluster,
    chain: &[ChainSpeaker],
    splays: &[f64],
) -> Result<RigPlan, ImpossibleConfiguration> {
    match cluster.rigging.points {
        1 => {}
        2 if cluster.pull_back_enabled => {
            return Err(ImpossibleConfiguration {
                reason: "Pull-back manuel et accroche 2 points ne se combinent pas : à 2 points, \
                         la charge se répartit déjà entre les deux moteurs."
                    .into(),
            });
        }
        2 => return Ok(RigPlan::Pair),
        n => {
            return Err(ImpossibleConfiguration {
                reason: format!("Accroche à {n} points non gérée : 1 ou 2 points seulement."),
            });
        }
    }
    if cluster.pull_back_enabled {
        if cluster.imposed_tilt.is_none() {
            return Err(ImpossibleConfiguration {
                reason: "Le pull-back manuel demande une assiette imposée : c'est avec \
                         elle que se répartit la charge entre le moteur principal et le \
                         pull-back."
                    .into(),
            });
        }
        return Ok(RigPlan::ManualPullBack);
    }
    let phi_of = |p: Vec2| phi_initial_free_hang(chain, splays, p);
    let target = cluster.imposed_tilt.map(f64::to_radians);
    let pick = pick_single(options, target, phi_of);
    // Pull-back imposé seulement si l'assiette visée sort de tout ce que les
    // trous permettent : entre deux trous, on accroche au plus proche et on
    // affiche l'écart.
    let forced_pull_back = target.is_some_and(|t| {
        let (lo, hi) = single_tilt_range(options, phi_of);
        (pick.phi - t).abs() > TILT_TOLERANCE_DEG.to_radians() && (t < lo || t > hi)
    });
    Ok(RigPlan::Single {
        group: pick.group,
        candidate: pick.candidate,
        forced_pull_back,
    })
}

struct RiggingViewInput<'a> {
    rig: RigSelection,
    options: &'a RiggingOptions,
    bar: Option<&'a BumperBarModel>,
    top: SpeakerInstance,
    /// Ce que la manille tire à un point, N, repère global.
    support_force: Vec2,
    gravity: f64,
    target_tilt_deg: Option<f64>,
    achieved_phi: f64,
    pull_back: bool,
}

fn rigging_view(input: RiggingViewInput) -> RiggingView {
    let RiggingViewInput { rig, options, bar, top, support_force, gravity, target_tilt_deg, achieved_phi, pull_back } = input;
    let to_global = |p: Vec2| top.o + p.rotate(top.phi);
    // Effort de chaque chaîne sur son trou : la manille unique tire selon
    // l'effort de support (incliné si un pull-back tire aussi) ; à deux points,
    // les deux chaînes sont verticales.
    let lifts: Vec<(Vec2, Vec2)> = rig
        .points
        .iter()
        .map(|(c, tension)| {
            let force = tension.map_or(support_force, |t| Vec2::new(0.0, t));
            (to_global(c.local), force)
        })
        .collect();
    let points = rig
        .points
        .into_iter()
        .map(|(c, tension)| {
            let tension_n = tension.unwrap_or(support_force.norm());
            let load_kg = tension_n / gravity;
            RiggingPointView {
                label: c.label,
                bumper_x_mm: c.bumper_x_mm,
                point_global: to_global(c.local),
                tension_n,
                load_kg,
                wll_kg: c.wll_kg,
                overloaded: load_kg > c.wll_kg,
            }
        })
        .collect();
    let (support, bar_mount_index) = match rig.group {
        GroupKind::Bumper => (RiggingSupport::Bumper, None),
        GroupKind::Bar { mount_index } => (RiggingSupport::Bar, Some(mount_index)),
    };
    // Barre dessinée telle que montée : ses trous et ses pattes, du repère
    // barre au repère global en passant par le bumper puis l'enceinte.
    let (bar_holes_global, bar_pins_global, bar_outline_global) =
        match (bar_mount_index, bar.and_then(|b| b.geometry.as_ref())) {
            (Some(i), Some(geometry)) => {
                let mount = options.mounts[i];
                let shift = options.speaker_half_height;
                let place = |p: [f64; 2]| to_global(mount.to_bumper(p) + Vec2::new(0.0, shift));
                (
                    geometry.pickup_holes.iter().map(|&p| place(p)).collect(),
                    geometry.link_pins.iter().map(|&p| place(p)).collect::<Vec<_>>(),
                    bar_outline_local(geometry).into_iter().map(place).collect(),
                )
            }
            _ => (Vec::new(), Vec::new(), Vec::new()),
        };
    // La barre est un corps rigide repris par ses deux goupilles : même
    // répartition élastique qu'une paire ancrage/verrou, chaque chaîne
    // ajoutant sa part. Pattes à même hauteur : les composantes verticales
    // retombent exactement sur la règle du levier.
    let bar_link_forces = match bar_pins_global.as_slice() {
        &[a, b] => {
            let (mut fa, mut fb) = (Vec2::ZERO, Vec2::ZERO);
            for &(at, force) in &lifts {
                let (da, db, _) = split_over_pair(a, b, at, force);
                fa = fa + da;
                fb = fb + db;
            }
            [(a, fa), (b, fb)]
                .into_iter()
                .map(|(point_global, force_global)| LinkForceView {
                    point_global,
                    force_global,
                    force_n: force_global.norm(),
                    angle_deg: angle_of(force_global.rotate_transpose(top.phi)),
                })
                .collect()
        }
        _ => Vec::new(),
    };
    let achieved_tilt_deg = achieved_phi.to_degrees();
    RiggingView {
        support,
        bar_mounts: options.mounts.iter().map(BarMountView::from).collect(),
        bar_mount_index,
        points,
        target_tilt_deg,
        achieved_tilt_deg,
        tilt_error_deg: target_tilt_deg.filter(|_| !pull_back).map(|t| achieved_tilt_deg - t),
        bumper_holes_global: options.bumper_holes_local.iter().map(|&p| to_global(p)).collect(),
        bumper_link_holes_global: options.link_holes_local.iter().map(|&p| to_global(p)).collect(),
        bar_holes_global,
        bar_pins_global,
        bar_outline_global,
        bar_link_forces,
    }
}

/// Calcule la géométrie, la statique de chaque jonction et la tension de pull-back
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
/// pas, un pull-back est automatiquement mis en place — sur le trou de couronne
/// splay 0 de l'enceinte du bas — pour tenir l'assiette exacte
/// malgré tout : ce n'est plus une case à cocher, c'est entièrement dérivé.
/// Le pull-back tire verticalement vers le haut : sa direction reste dans
/// 180° ± `Settings::pull_back_tolerance_deg`, rognée aux directions où le
/// câble tire (`pull_back_usable_range_deg`). L'angle exact reste un choix
/// utilisateur dans cette plage (`Cluster::pull_back_angle`), la verticale étant
/// suggérée tant qu'il n'a pas choisi.
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

    // Trous d'accroche déclarés : en vol seulement, et seulement si le bumper
    // les cote. Sinon, ancienne accroche continue.
    let rigging = match compartment {
        Compartment::Stacked => None,
        Compartment::Flown => rigging_options(
            bumper_model,
            active_bumper_bar,
            reference_model.mechanical.height / 2.0,
            &cluster.rigging,
        )
        .transpose()?,
    };
    let rig_plan = rigging
        .as_ref()
        .map(|options| plan_rigging(options, cluster, chain, &splays))
        .transpose()?;

    let phi_initial = match compartment {
        Compartment::Stacked => {
            let bottom_deg = cluster.imposed_tilt.unwrap_or(0.0);
            phi_initial_stack(bottom_deg, &splays)
        }
        Compartment::Flown => match cluster.imposed_tilt {
            // À un point sans pull-back, l'assiette est celle que donne le
            // trou retenu, pas celle demandée : l'écart s'affiche.
            _ if matches!(rig_plan, Some(RigPlan::Single { forced_pull_back: false, .. })) => {
                let Some(RigPlan::Single { candidate, .. }) = &rig_plan else { unreachable!() };
                phi_initial_free_hang(chain, &splays, candidate.local)
            }
            // Deux points sans assiette demandée : à plat.
            None if matches!(rig_plan, Some(RigPlan::Pair)) => 0.0,
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

    // Poids et CG de l'ensemble, une fois pour toutes : le pull-back ne
    // s'intéresse qu'à la résultante, pas à la répartition enceinte par
    // enceinte (qui, elle, compte pour chaque jonction).
    let total_mass_kg: f64 = chain.iter().map(|c| c.mass_kg).sum();
    let total_weight_n = total_mass_kg * settings.gravity * settings.dynamic_factor;
    let cluster_cg = weighted_cg(chain, &speakers);

    // Point d'accroche final, et pull-back. Deux façons d'en avoir un :
    // * **imposé** : le bumper (+ sa barre) ne suffit plus à atteindre seul
    //   l'assiette voulue ; l'accroche est plafonnée au bout de la barre ;
    // * **manuel** (`Cluster::pull_back_enabled`) : l'assiette tient sans lui,
    //   mais l'utilisateur veut répartir la charge entre le moteur principal et
    //   le pull-back, parce que les points d'accroche sont faibles. La
    //   direction et la tension sont alors des saisies
    //   (`Cluster::manual_pull_back_tension_n`) ; l'accroche en découle.
    let flown = match compartment {
        Compartment::Stacked => FlownSupport::centered(Vec2::ZERO),
        Compartment::Flown if rig_plan.is_some() => {
            let options = rigging.as_ref().expect("plan sans options");
            let pull_back_point = chain[chain.len() - 1].geo.crown(0.0);
            let resolve = |pickup: Vec2, requested_angle: Option<f64>| {
                resolve_pull_back(&speakers, total_weight_n, cluster_cg, pull_back_point, pickup, requested_angle, settings)
            };
            match rig_plan.expect("vérifié ci-dessus") {
                RigPlan::Single { group, candidate, forced_pull_back } => {
                    let pickup = candidate.local;
                    let rig = Some(RigSelection { group, points: vec![(candidate, None)] });
                    if forced_pull_back {
                        let (force, range) = resolve(pickup, cluster.pull_back_angle)?;
                        FlownSupport {
                            pickup,
                            pull_back_forced: true,
                            pull_back: Some(force),
                            angle_range: Some(range),
                            tension_range: None,
                            rig,
                        }
                    } else {
                        FlownSupport { rig, ..FlownSupport::centered(pickup) }
                    }
                }
                RigPlan::ManualPullBack => {
                    // Direction dans la fenêtre verticale ; chaque trou donne
                    // sa tension, on retient celui qui approche la demande.
                    let window = pull_back_window_deg(settings.pull_back_tolerance_deg);
                    let angle = cluster
                        .pull_back_angle
                        .map(|a| clamp_to_pull_back_range(a, window))
                        .unwrap_or(PULL_BACK_VERTICAL_DEG);
                    let tensioned: Vec<(GroupKind, &Candidate, f64)> = options
                        .groups
                        .iter()
                        .flat_map(|g| g.candidates.iter().map(move |c| (g.kind, c)))
                        .map(|(kind, c)| {
                            let t = pull_back_tension(&speakers, total_weight_n, cluster_cg, c.local, pull_back_point, angle);
                            (kind, c, t)
                        })
                        .filter(|&(_, _, t)| t.is_finite() && t > 0.0)
                        .collect();
                    if tensioned.is_empty() {
                        return Err(ImpossibleConfiguration {
                            reason: format!(
                                "Pull-back manuel impossible : à cette assiette, aucun trou d'accroche du bumper ou de sa barre ne met un pull-back à {angle:.1}° en traction."
                            ),
                        });
                    }
                    let tension_range = tensioned
                        .iter()
                        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(_, _, t)| (lo.min(t), hi.max(t)));
                    let half_load = 0.5 * total_weight_n / dir_from_angle(angle).y;
                    let target = cluster
                        .manual_pull_back_tension_n
                        .unwrap_or(half_load)
                        .clamp(tension_range.0, tension_range.1);
                    // `min_by` garde le premier à égalité : l'ordre des familles
                    // (bumper d'abord) départage.
                    let &(group, candidate, _) = tensioned
                        .iter()
                        .min_by(|a, b| (a.2 - target).abs().total_cmp(&(b.2 - target).abs()))
                        .expect("non vide");
                    let (force, _) = resolve(candidate.local, Some(angle))?;
                    FlownSupport {
                        pickup: candidate.local,
                        pull_back_forced: false,
                        pull_back: Some(force),
                        angle_range: Some(window),
                        tension_range: Some(tension_range),
                        rig: Some(RigSelection { group, points: vec![(candidate.clone(), None)] }),
                    }
                }
                RigPlan::Pair => {
                    let top = speakers[0];
                    let pick = pick_pair(options, cluster_cg.x, total_weight_n, settings.gravity, |p| {
                        (top.o + p.rotate(top.phi)).x
                    })
                    .ok_or_else(|| ImpossibleConfiguration {
                        reason: "Accroche 2 points impossible : aucune paire de trous du bumper ou de la barre n'encadre le centre de gravité de la grappe à cette assiette. Essaie la barre, un autre montage, ou réduis l'assiette.".into(),
                    })?;
                    // Le point le plus chargé tient lieu d'accroche principale
                    // pour les champs hérités (dessin, altitude).
                    let [(a, ta), (b, tb)] = pick.points;
                    let pickup = if ta >= tb { a.local } else { b.local };
                    FlownSupport {
                        rig: Some(RigSelection {
                            group: pick.group,
                            points: vec![(a, Some(ta)), (b, Some(tb))],
                        }),
                        ..FlownSupport::centered(pickup)
                    }
                }
            }
        }
        Compartment::Flown => {
            let pickup_height = bumper_pickup_height(reference_model, bumper_model);
            match cluster.imposed_tilt {
                None if cluster.pull_back_enabled => {
                    return Err(ImpossibleConfiguration {
                        reason: "Le pull-back manuel demande une assiette imposée : c'est avec \
                                 elle que se répartit la charge entre le moteur principal et le \
                                 pull-back."
                            .into(),
                    });
                }
                None => FlownSupport::centered(Vec2::new(0.0, pickup_height)),
                Some(_) => {
                    let raw_x =
                        solve_pickup_x_for_imposed_tilt(chain, &splays, phi_initial, pickup_height);
                    // Sans barre compatible, la zone de fixation directe est la
                    // seule portée disponible : au-delà, il faut déjà un pull-back.
                    let bumper_bar_max = active_bumper_bar
                        .map(|bumper_bar| bumper_bar.max_deport_mm)
                        .unwrap_or(bumper_model.max_direct_deport_mm);
                    // Trou de couronne 0° de l'enceinte du bas.
                    let pull_back_point = chain[chain.len() - 1].geo.crown(0.0);
                    let tension_at = |pickup: Vec2, angle: f64| {
                        pull_back_tension(
                            &speakers,
                            total_weight_n,
                            cluster_cg,
                            pickup,
                            pull_back_point,
                            angle,
                        )
                    };
                    // Direction, tension et test de collision, communs aux deux
                    // modes une fois l'accroche fixée.
                    let resolve = |pickup: Vec2, requested_angle: Option<f64>| {
                        resolve_pull_back(&speakers, total_weight_n, cluster_cg, pull_back_point, pickup, requested_angle, settings)
                    };

                    if raw_x.abs() > bumper_bar_max {
                        // Pull-back imposé : l'accroche est plafonnée au bout
                        // de la barre, le pull-back tient le reste.
                        let pickup = Vec2::new(bumper_bar_max * raw_x.signum(), pickup_height);
                        let (force, range) = resolve(pickup, cluster.pull_back_angle)?;
                        FlownSupport {
                            pickup,
                            pull_back_forced: true,
                            pull_back: Some(force),
                            angle_range: Some(range),
                            tension_range: None,
                            rig: None,
                        }
                    } else if cluster.pull_back_enabled {
                        // Pull-back manuel : l'utilisateur fixe la direction
                        // et la tension ; l'accroche en découle par l'équilibre
                        // des moments autour d'elle. Direction dans la fenêtre
                        // verticale, tension bornée par les positions
                        // d'accroche atteignables sur le bumper et sa barre.
                        let window = pull_back_window_deg(settings.pull_back_tolerance_deg);
                        let angle = cluster
                            .pull_back_angle
                            .map(|a| clamp_to_pull_back_range(a, window))
                            .unwrap_or(PULL_BACK_VERTICAL_DEG);
                        let at = |x: f64| tension_at(Vec2::new(x, pickup_height), angle);
                        let Some(pickup_range) = manual_pickup_range_mm(raw_x, bumper_bar_max, at) else {
                            return Err(ImpossibleConfiguration {
                                reason: format!(
                                    "Pull-back manuel impossible : à cette assiette, aucune position d'accroche sur le bumper ou sa barre ne met un pull-back à {angle:.1}° en traction."
                                ),
                            });
                        };
                        let (t_a, t_b) = (at(pickup_range.0), at(pickup_range.1));
                        let tension_range = (t_a.min(t_b), t_a.max(t_b));
                        // Sans saisie : la moitié de la charge au pull-back.
                        let half_load = 0.5 * total_weight_n / dir_from_angle(angle).y;
                        let target = cluster
                            .manual_pull_back_tension_n
                            .unwrap_or(half_load)
                            .clamp(tension_range.0, tension_range.1);
                        let x = pickup_for_tension_mm(pickup_range, target, at);
                        let pickup = Vec2::new(x, pickup_height);
                        let (force, _) = resolve(pickup, Some(angle))?;
                        FlownSupport {
                            pickup,
                            pull_back_forced: false,
                            pull_back: Some(force),
                            angle_range: Some(window),
                            tension_range: Some(tension_range),
                            rig: None,
                        }
                    } else {
                        FlownSupport::centered(Vec2::new(raw_x, pickup_height))
                    }
                }
            }
        }
    };
    let pickup = flown.pickup;
    let flown_rig = flown.rig;
    let bumper_bar_exceeded = flown.pull_back_forced;
    let pull_back_angle_range_deg = flown.angle_range;

    // Le pull-back n'existe que s'il est imposé ou demandé ci-dessus. Sa
    // direction reste un choix utilisateur dans la plage affichée
    // (`pull_back_angle_range_deg`) — l'algorithme délimite, ne décide pas.
    let pull_back_force = flown.pull_back;
    let tension = pull_back_force.map(|t| t.force.norm()).unwrap_or(0.0);

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
                pull_back: pull_back_force,
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

    let pull_back_point_global = pull_back_force.map(|t| {
        let last = speakers[speakers.len() - 1];
        last.o + t.point_local.rotate(last.phi)
    });
    let pull_back_direction_global = pull_back_force.map(|t| t.force.normalize());
    let pull_back_direction_angle_deg = pull_back_direction_global.map(angle_of);

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
            // En vol, le bumper est calé par ses deux liaisons réelles — la
            // bielle avant et la barre arrière — et non par le perçage coté
            // depuis ses bords (`bumper_pin_points`), qui reste le tracé du
            // dessin et le placement du stack.
            //
            // En vol, c'est la barre d'aplomb qui est montée : les barres
            // inclinées servent à pencher la première tête en stack.
            let rear_bar =
                bumper_model
                    .rear_bar_at_tilt(0.0)
                    .ok_or_else(|| ImpossibleConfiguration {
                        reason: format!(
                            "Le bumper {} ne déclare pas de barre arrière à 0° : \
                             impossible de le monter en vol.",
                            bumper_model.name
                        ),
                    })?;
            let loads = compute_bumper_loads(
                chain,
                &speakers,
                bumper_model.pivot_bar_usable_length_mm,
                rear_bar,
                settings,
                pull_back_force,
            )?;
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
                pull_back_angle_range_deg: pull_back_angle_range_deg.map(|(lo, hi)| [lo, hi]),
                pull_back_tension_range_n: flown.tension_range.map(|(lo, hi)| [lo, hi]),
                pull_back_load_share: pull_back_force
                    .map(|f| f.force.y / total_weight_n)
                    .unwrap_or(0.0),
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
                rigging: flown_rig.map(|rig| {
                    rigging_view(RiggingViewInput {
                        rig,
                        options: rigging.as_ref().expect("sélection sans options"),
                        bar: active_bumper_bar,
                        top: attach,
                        support_force: loads.support_force,
                        gravity: settings.gravity,
                        target_tilt_deg: cluster.imposed_tilt,
                        achieved_phi: phi_initial,
                        pull_back: pull_back_force.is_some(),
                    })
                }),
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
                pull_back_angle_range_deg: None,
                pull_back_tension_range_n: None,
                pull_back_load_share: 0.0,
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
                rigging: None,
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
        pull_back_tension_n: tension,
        joints,
        cg: cluster_cg,
        total_mass_kg,
        speaker_names: resolved.models.iter().map(|m| m.name.clone()).collect(),
        pickup_global,
        pull_back_point_global,
        pull_back_direction_global,
        pull_back_direction_angle_deg,
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

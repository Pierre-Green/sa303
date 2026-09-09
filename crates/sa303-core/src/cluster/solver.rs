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
use crate::bumper::{bumper_outline_top, bumper_pickup_height, BumperBarModel, BumperModel};
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
    /// cette enceinte — même schéma qu'une jonction réelle (bras à deux
    /// forces + pivot), avec pour seuls points d'accroche le trou de splay 0
    /// et la charnière. Vol uniquement.
    pub orientation_force_n: Option<f64>,
    pub orientation_angle_deg: Option<f64>,
    pub pivot_force_n: Option<f64>,
    pub pivot_angle_deg: Option<f64>,
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

/// Efforts transmis par le bumper à l'enceinte de référence (`speakers[0]`),
/// exprimés dans le repère de cette enceinte — même schéma qu'une jonction
/// réelle (brief §5) : un bras à deux forces entre le trou de splay 0
/// (`anchor_at(0)`, côté enceinte) et le point d'accroche (côté bumper), et
/// un pivot au HT de l'enceinte. Corps libre = toutes les enceintes, puisque
/// tout pend de ce point.
fn compute_bumper_loads(
    chain: &[ChainSpeaker],
    speakers: &[SpeakerInstance],
    pickup_local: Vec2,
    g: f64,
    k_dyn: f64,
    share_per_flank: f64,
    tie: Option<TieForce>,
) -> (f64, f64, f64, f64) {
    // Le bumper est posé sur l'enceinte du haut : c'est sa quincaillerie à
    // elle qui reprend l'effort.
    let geo = chain[0].geo;
    let b0 = speakers[0];
    let pvg = b0.o + geo.ht.rotate(b0.phi);
    let an_local = geo.anchor_at(0.0);
    let bo_local = Vec2::new(an_local.x, pickup_local.y);
    let bo = b0.o + bo_local.rotate(b0.phi);
    let an = b0.o + an_local.rotate(b0.phi);

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
    let mut mext = (cm - pvg).cross(rext);
    if let Some(tie) = tie {
        let last = speakers[speakers.len() - 1];
        let q = last.o + tie.point_local.rotate(last.phi);
        rext = rext + tie.force;
        mext += (q - pvg).cross(tie.force);
    }

    let u = (bo - an).normalize();
    let lever = (an - pvg).cross(u);
    let lambda = -mext / lever;
    let f_ori = u * lambda;
    let f_piv = -rext - f_ori;

    let sg = -share_per_flank;
    let f_ori_local = (f_ori * sg).rotate_transpose(b0.phi);
    let f_piv_local = (f_piv * sg).rotate_transpose(b0.phi);

    (
        f_ori_local.norm(),
        angle_of(f_ori_local),
        f_piv_local.norm(),
        angle_of(f_piv_local),
    )
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
                        // Point 0° arrière-bas de l'enceinte du bas — même
                        // référence que côté bumper (anchor_at(0)), pas la
                        // charnière avant.
                        // Point 0° de l'enceinte du BAS : c'est elle qui porte
                        // la tirette, pas celle du haut.
                        let tie_point = chain[chain.len() - 1].geo.anchor_at(0.0);
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
        .collect();

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
            let (of_n, of_a, pf_n, pf_a) = compute_bumper_loads(
                chain,
                &speakers,
                pickup,
                settings.gravity,
                settings.dynamic_factor,
                settings.share_per_flank,
                tie_force,
            );
            BumperView {
                outline_global,
                pickup_global: Some(attach.o + pickup.rotate(attach.phi)),
                bumper_bar_start_global,
                pickup_offset_mm: Some(pickup.x),
                bar_deport_mm: Some(deport),
                bumper_bar_exceeded,
                tie_angle_range_deg: tie_angle_range_deg.map(|(lo, hi)| [lo, hi]),
                orientation_force_n: Some(of_n),
                orientation_angle_deg: Some(of_a),
                pivot_force_n: Some(pf_n),
                pivot_angle_deg: Some(pf_a),
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
            BumperView {
                outline_global,
                pickup_global: None,
                bumper_bar_start_global: None,
                pickup_offset_mm: None,
                bar_deport_mm: None,
                bumper_bar_exceeded: false,
                tie_angle_range_deg: None,
                orientation_force_n: None,
                orientation_angle_deg: None,
                pivot_force_n: None,
                pivot_angle_deg: None,
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

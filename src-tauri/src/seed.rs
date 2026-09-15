//! Catalogue de base embarqué dans le logiciel (brief §3, §6, §8).
//!
//! Enceintes, bumpers, barres et grappes ne sont plus construits en Rust mais
//! lus depuis `src-tauri/assets/`, un fichier JSON par élément, au format exact
//! de ceux qu'écrit l'application. Trois conséquences voulues :
//!
//! - un composant se conçoit dans l'éditeur puis se dépose ici, sans code ;
//! - le format des assets est par construction celui des fichiers réels, il ne
//!   peut donc pas dériver en silence ;
//! - une mise à jour du logiciel remplace ces éléments dans le dossier de
//!   données sans qu'il faille l'effacer (voir `persistence::sync_builtins`).
//!
//! Ces éléments sont **immuables côté application** : la persistance refuse de
//! les enregistrer ou de les supprimer. Seule une mise à jour les fait bouger.
//!
//! Les réglages (`default_settings`) restent du code : ce ne sont pas des
//! composants du catalogue mais des préférences que l'utilisateur ajuste.

use crate::persistence::migrate_legacy_cluster_json;
use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::settings::{AxisMapping, PinSpec, PlateSpec, Settings};
use sa303_core::speaker::SpeakerModel;
use serde::de::DeserializeOwned;

// Tables des assets embarqués, produites par `build.rs` à partir du contenu
// réel des dossiers.
include!(concat!(env!("OUT_DIR"), "/assets.rs"));

/// Un élément du catalogue de base : son identifiant (le nom du fichier) et le
/// modèle lu. L'identifiant est repris du nom de fichier plutôt que du JSON :
/// c'est lui qui nomme le fichier écrit dans le dossier de données, donc les
/// deux ne peuvent pas diverger.
pub struct Builtin<T> {
    pub id: String,
    pub model: T,
}

/// Lit une famille d'assets. Un asset illisible est une erreur de
/// développement, pas une situation d'exécution : le fichier est embarqué dans
/// le binaire, donc s'il est cassé il l'est pour tout le monde et il vaut mieux
/// le savoir au premier lancement.
fn parse_assets<T: DeserializeOwned>(
    assets: &[(&str, &str)],
    kind: &str,
    prepare: fn(serde_json::Value) -> serde_json::Value,
) -> Vec<Builtin<T>> {
    assets
        .iter()
        .map(|(id, raw)| {
            let value: serde_json::Value = serde_json::from_str(raw)
                .unwrap_or_else(|e| panic!("asset {kind}/{id}.json : JSON invalide : {e}"));
            let model = serde_json::from_value(prepare(value))
                .unwrap_or_else(|e| panic!("asset {kind}/{id}.json illisible : {e}"));
            Builtin {
                id: (*id).to_string(),
                model,
            }
        })
        .collect()
}

fn as_is(value: serde_json::Value) -> serde_json::Value {
    value
}

pub fn builtin_speakers() -> Vec<Builtin<SpeakerModel>> {
    parse_assets(&SPEAKER_ASSETS, "speakers", as_is)
}

pub fn builtin_bumpers() -> Vec<Builtin<BumperModel>> {
    parse_assets(&BUMPER_ASSETS, "bumpers", as_is)
}

pub fn builtin_bumper_bars() -> Vec<Builtin<BumperBarModel>> {
    parse_assets(&BUMPER_BAR_ASSETS, "bumper-bars", as_is)
}

/// Les grappes passent par la même migration que les fichiers du disque : un
/// exemple déposé avant un changement de schéma reste lisible.
pub fn builtin_clusters() -> Vec<Builtin<Cluster>> {
    parse_assets(&CLUSTER_ASSETS, "clusters", migrate_legacy_cluster_json)
}

// Perçage de référence de l'accastillage SA303. Il ne sert plus à construire
// l'enceinte — c'est l'asset qui fait foi — mais à le vérifier : un test
// confronte la grille livrée à ces deux arcs, donc une faute de frappe dans le
// JSON ne peut pas passer.
/// Trous de réglage d'orientation percés sur l'arc court, c'est-à-dire la
/// couronne intérieure (rayon `radius − delta`). Ce sont les splays impairs :
/// c'est la convention que `speaker::geometry::is_odd_splay` applique pour
/// choisir le rayon, donc les deux doivent rester d'accord.
#[cfg(test)]
pub const SA303_SHORT_ARC_SPLAYS: [f64; 3] = [1.0, 3.0, 5.0];

/// Trous percés sur l'arc long, la couronne extérieure (rayon `radius`) : les
/// splays pairs.
#[cfg(test)]
pub const SA303_LONG_ARC_SPLAYS: [f64; 5] = [0.0, 2.0, 4.0, 10.0, 20.0];

/// Les angles réellement disponibles sur l'accastillage, les deux arcs réunis
/// et triés. Un angle absent de cette liste n'a pas de trou : la jonction est
/// alors impossible, jamais approchée en silence (brief §11.6).
#[cfg(test)]
pub fn sa303_splay_grid() -> Vec<f64> {
    let mut grid: Vec<f64> = SA303_SHORT_ARC_SPLAYS
        .iter()
        .chain(SA303_LONG_ARC_SPLAYS.iter())
        .copied()
        .collect();
    grid.sort_by(f64::total_cmp);
    grid
}

pub fn default_settings() -> Settings {
    Settings {
        safety_factor: 4.0,
        dynamic_factor: 1.3,
        gravity: 9.80665,
        share_per_flank: 0.5,
        pin: PinSpec {
            diameter: 12.0,
            ultimate: 1000.0,
            net_section: 0.86,
        },
        plate: PlateSpec {
            flank_thickness: 4.0,
            bar_thickness: 10.0,
            ultimate: 510.0,
        },
        axis_mapping: AxisMapping {
            tool_x: "X".into(),
            tool_y: "-Y".into(),
        },
    }
}

/// Tableau de charges par jonction sur le catalogue de grappes livré, avec
/// balayage de la largeur de barre. Ce n'est pas un test — rien n'y est asserté
/// — mais le moyen de régénérer le tableau du dossier d'audit sur les grappes
/// réelles plutôt que sur le jeu synthétique de `sa303-core` :
///
/// ```text
/// cargo test -p sa303 dump_catalogue_load_table -- --ignored --nocapture
/// ```
#[cfg(test)]
#[test]
#[ignore]
fn dump_catalogue_load_table() {
    use sa303_core::checks::{check_bar, utilization, SandwichSpec};
    use sa303_core::compute_cluster;

    let speakers: Vec<_> = builtin_speakers().into_iter().map(|b| b.model).collect();
    let bumpers: Vec<_> = builtin_bumpers().into_iter().map(|b| b.model).collect();
    let bars: Vec<_> = builtin_bumper_bars().into_iter().map(|b| b.model).collect();
    let clusters: Vec<_> = builtin_clusters().into_iter().map(|b| b.model).collect();
    let settings = default_settings();
    let spec = SandwichSpec::from_settings(&settings);

    // Les quatre grappes demandées, repérées par un fragment de leur nom.
    let wanted = ["40d-88d", "-20d-46d", "0-150m", "0-60m"];

    for width in [40.0_f64, 70.0] {
        println!("=== largeur de barre {width} mm ===");
        println!("grappe|J|splay|N|V|Mmax|sigma|ou|larg|Fverrou|Fancrage|couronne|bielle|verrou|ancrage|barre|arrach|pire");
        for cluster in &clusters {
            if !wanted.iter().any(|w| cluster.name.contains(w)) {
                continue;
            }
            // Le balayage se fait sur le modèle, pas dans le code de calcul :
            // c'est bien la cote de la pièce qu'on fait varier.
            //
            // Élargir la partie courante sans bouger la partie élargie
            // produirait une pièce absurde — un « large » plus étroit que
            // l'« étroit » — et surtout ramènerait le bord avant sur `up660`,
            // qui vit à 19,4 mm de l'axe. Le bord avant est donc tenu fixe :
            // l'élargissement se fait d'un seul côté, vers l'arrière, et
            // `e₂ up660` reste à ses 15,6 mm.
            let front_edge = 55.0 - 40.0 / 2.0;
            let swept: Vec<_> = speakers
                .iter()
                .cloned()
                .map(|mut s| {
                    s.mechanical.rear_bar.narrow_width = width;
                    s.mechanical.rear_bar.wide_width = front_edge + width / 2.0;
                    s
                })
                .collect();
            let Some(bumper) = bumpers.iter().find(|b| b.id == cluster.bumper_model_id) else {
                println!("{}|bumper introuvable", cluster.name);
                continue;
            };
            match compute_cluster(&swept, cluster, &settings, bumper, &bars) {
                Err(e) => println!("{}|IMPOSSIBLE|{}", cluster.name, e.reason),
                Ok(r) => {
                    for j in &r.joints {
                        let b = check_bar(
                            &j.rear_bar,
                            j.splay_deg,
                            j.bar_shear_n,
                            j.bar_axial_n,
                            settings.safety_factor,
                        );
                        let w = b.worst_section();
                        let uo = utilization(j.f_orientation_n, &spec);
                        let up = utilization(j.f_pivot_n, &spec);
                        let ua = utilization(j.f_anchor_n, &spec);
                        let ul = utilization(j.f_latch_n, &spec);
                        let ub = w.utilization;
                        let ut = b.worst_tear_out().utilization;
                        println!(
                            "{}|{}|{}|{:.0}|{:.0}|{:.1}|{:.1}|{}|{:.0}|{:.0}|{:.0}|{:.3}|{:.3}|{:.3}|{:.3}|{:.3}|{:.3}|{:.3}",
                            cluster.name, j.joint_index + 1, j.splay_deg,
                            j.bar_axial_n, j.bar_shear_n, j.bar_moment_max_nm,
                            w.stress_mpa, w.location, b.critical_width_mm,
                            j.f_latch_n, j.f_anchor_n,
                            uo, up, ul, ua, ub, ut,
                            uo.max(up).max(ua).max(ul).max(ub).max(ut)
                        );
                    }
                }
            }
        }
    }
}

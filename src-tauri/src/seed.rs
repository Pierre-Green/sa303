//! Enceinte, réglages et grappes par défaut (brief §3, §6, §8), embarqués au
//! premier lancement.

use crate::persistence::SPEAKER_SCHEMA_VERSION;
use sa303_core::bumper::{
    BumperBarCompatibility, BumperBarModel, BumperCompatibility, BumperModel,
};
use sa303_core::cluster::{Cluster, Compartment, JointSetting};
use sa303_core::settings::{AxisMapping, PinSpec, PlateSpec, Settings};
use sa303_core::speaker::{
    BelowCompatibility, Crown, Hinge, SpeakerAcousticsModel, SpeakerMechanicalModel, SpeakerModel,
    SplayRange, WaveguideFront,
};

pub const SPEAKER_ISOPHASE_ID: &str = "sa303-isophase";
pub const SPEAKER_CCA_ID: &str = "sa303-cca";
pub const DEFAULT_BUMPER_ID: &str = "sa303-bumper";
pub const DEFAULT_BUMPER_BAR_ID: &str = "sa303-bumper-bar";
/// Hauteur d'accroche des grappes d'exemple, mm : une valeur de plein air
/// courante, pour que la mise en situation soit visible dès le premier
/// lancement plutôt que d'avoir tout au ras du sol.
const DEFAULT_TRIM_HEIGHT_MM: f64 = 8000.0;

/// Mécanique commune aux deux SA303 : même châssis, donc même quincaillerie
/// de rigging. C'est ce qui les rend assemblables l'un sous l'autre — mais ce
/// sont bien deux enceintes distinctes, pas deux variantes d'un même modèle :
/// un futur renfort de grave aura sa propre masse et son propre CG.
fn sa303_mechanical() -> SpeakerMechanicalModel {
    SpeakerMechanicalModel {
        depth: 700.0,
        height: 550.0,
        total_vertical_angle: 20.0,
        mass_kg: 83.695,
        cg: [10.84, 11.91],
        hinge: Hinge {
            x: -338.43,
            y: 257.13,
            joint_separation: 552.384,
        },
        crown: Crown {
            radius: 680.0,
            delta: 20.0,
            anchor_angle: 2.5,
            splay0_angle: 5.0,
        },
        splay_grid: sa303_splay_grid(),
        frame_hole_splay: 0.0,
    }
}

/// Trous de réglage d'orientation percés sur l'arc court, c'est-à-dire la
/// couronne intérieure (rayon `radius − delta`). Ce sont les splays impairs :
/// c'est la convention que `speaker::geometry::is_odd_splay` applique pour
/// choisir le rayon, donc les deux doivent rester d'accord.
pub const SA303_SHORT_ARC_SPLAYS: [f64; 8] = [1.0, 3.0, 5.0, 9.0, 11.0, 15.0, 17.0, 19.0];

/// Trous percés sur l'arc long, la couronne extérieure (rayon `radius`) : les
/// splays pairs.
pub const SA303_LONG_ARC_SPLAYS: [f64; 9] = [0.0, 2.0, 4.0, 8.0, 10.0, 12.0, 16.0, 18.0, 20.0];

/// Les angles réellement disponibles sur l'accastillage, les deux arcs réunis
/// et triés. Un angle absent de cette liste n'a pas de trou : la jonction est
/// alors impossible, jamais approchée en silence (brief §11.6).
pub fn sa303_splay_grid() -> Vec<f64> {
    let mut grid: Vec<f64> = SA303_SHORT_ARC_SPLAYS
        .iter()
        .chain(SA303_LONG_ARC_SPLAYS.iter())
        .copied()
        .collect();
    grid.sort_by(f64::total_cmp);
    grid
}

/// SA303-ISOPHASE : tête à guide d'onde isophase, la partie haute d'une
/// grappe. Sous elle : une autre ISOPHASE resserrée (1-4°), ou le passage en
/// CCA à 10° quand on commence à ouvrir vers le bas.
pub fn speaker_isophase() -> SpeakerModel {
    SpeakerModel {
        id: SPEAKER_ISOPHASE_ID.into(),
        name: "SA303-ISOPHASE".into(),
        schema_version: SPEAKER_SCHEMA_VERSION,
        mechanical: sa303_mechanical(),
        acoustics: SpeakerAcousticsModel {
            fs: 60.0,
            directivity_horizontal: 90.0,
            // Front isophase : pas de secteur propre — c'est le splay
            // mécanique qui ouvre la ligne.
            directivity_vertical: 0.0,
            wg_front: WaveguideFront::Isophase,
            wg_output_height: 464.0,
            // Sans objet sur un front plan : conservé au défaut.
            wg_level_at_half_coverage_db: -6.0,
        },
        compatible_below: vec![
            BelowCompatibility {
                speaker_model_id: SPEAKER_ISOPHASE_ID.into(),
                flown: true,
                stacked: true,
                recommended_splay: Some(SplayRange {
                    min_deg: 0.0,
                    max_deg: 4.0,
                }),
            },
            BelowCompatibility {
                speaker_model_id: SPEAKER_CCA_ID.into(),
                flown: true,
                stacked: true,
                recommended_splay: Some(SplayRange::exactly(10.0)),
            },
        ],
    }
}

/// SA303-CCA : tête à couverture large. Elle accepte aussi bien une CCA (20°)
/// qu'une ISOPHASE (10°) en dessous : mécaniquement rien ne l'interdit, c'est
/// une question d'acoustique — hors de l'angle recommandé, la jonction est
/// seulement signalée, jamais refusée.
pub fn speaker_cca() -> SpeakerModel {
    SpeakerModel {
        id: SPEAKER_CCA_ID.into(),
        name: "SA303-CCA".into(),
        schema_version: SPEAKER_SCHEMA_VERSION,
        mechanical: sa303_mechanical(),
        acoustics: SpeakerAcousticsModel {
            fs: 60.0,
            directivity_horizontal: 90.0,
            directivity_vertical: 20.0,
            wg_front: WaveguideFront::ConstantCurvature,
            wg_output_height: 464.0,
            wg_level_at_half_coverage_db: -6.0,
        },
        compatible_below: vec![
            BelowCompatibility {
                speaker_model_id: SPEAKER_CCA_ID.into(),
                flown: true,
                stacked: true,
                recommended_splay: Some(SplayRange::exactly(20.0)),
            },
            BelowCompatibility {
                speaker_model_id: SPEAKER_ISOPHASE_ID.into(),
                flown: true,
                stacked: true,
                recommended_splay: Some(SplayRange::exactly(10.0)),
            },
        ],
    }
}

pub fn default_speaker_models() -> Vec<SpeakerModel> {
    vec![speaker_isophase(), speaker_cca()]
}

/// SA303-BUMPER : bumper de capotage compatible avec les deux SA303 (même
/// châssis, donc même interface), en vol comme en stack. Tube 100×50, la manille se
/// ferme ~40 mm au-dessus du dessus du bumper. La SA303-BUMPER-BAR (barre de
/// déport) n'entre pas dans ce modèle : ses perçages ne sont pas à hauteur
/// constante, seul le bumper porte la géométrie — la barre n'est qu'un rendu
/// dérivé.
pub fn default_bumper_model() -> BumperModel {
    BumperModel {
        id: DEFAULT_BUMPER_ID.into(),
        name: "SA303-BUMPER".into(),
        schema_version: 1,
        depth: 702.0,
        height: 100.0,
        shackle_height_above_bumper: 40.0,
        max_direct_deport_mm: 702.0 / 2.0,
        compatible_speakers: vec![
            BumperCompatibility {
                speaker_model_id: SPEAKER_ISOPHASE_ID.into(),
                flown: true,
                stacked: true,
            },
            BumperCompatibility {
                speaker_model_id: SPEAKER_CCA_ID.into(),
                flown: true,
                stacked: true,
            },
        ],
    }
}

/// SA303-BUMPER-BAR : barre de déport, composant d'équipement à part
/// entière, jamais modélisée géométriquement (brief, cf. `BumperBarModel`) —
/// seule sa portée max entre dans le solveur.
pub fn default_bumper_bar_model() -> BumperBarModel {
    BumperBarModel {
        id: DEFAULT_BUMPER_BAR_ID.into(),
        name: "SA303-BUMPER-BAR".into(),
        schema_version: 1,
        max_deport_mm: 1500.0,
        compatible_bumpers: vec![BumperBarCompatibility {
            bumper_model_id: DEFAULT_BUMPER_ID.into(),
        }],
    }
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

fn joints(splays: &[f64]) -> Vec<JointSetting> {
    splays.iter().map(|&splay| JointSetting { splay }).collect()
}

/// Démontre le mélange d'enceintes dans une même grappe (brief) :
/// SA303-ISOPHASE en haut, bascule en SA303-CCA à partir du premier splay de
/// +10° (inclus) et jusqu'au bas — exactement la jonction pour laquelle
/// `speaker_isophase` recommande 10°. `splays` va du haut vers le bas, comme
/// `Cluster::joints`.
fn speaker_model_ids_for(splays: &[f64]) -> Vec<String> {
    let speaker_count = splays.len() + 1;
    let switch_at = splays
        .iter()
        .position(|&s| s == 10.0)
        .map(|joint_idx| joint_idx + 1);
    (0..speaker_count)
        .map(|index| match switch_at {
            Some(from) if index >= from => SPEAKER_CCA_ID.into(),
            _ => SPEAKER_ISOPHASE_ID.into(),
        })
        .collect()
}

fn flown(id: &str, name: &str, splays: &[f64]) -> Cluster {
    Cluster {
        id: id.into(),
        name: name.into(),
        schema_version: 1,
        speaker_model_ids: speaker_model_ids_for(splays),
        compartment: Compartment::Flown,
        joints: joints(splays),
        imposed_tilt: None,
        tie_angle: None,
        bumper_model_id: DEFAULT_BUMPER_ID.into(),
        bumper_height: DEFAULT_TRIM_HEIGHT_MM,
    }
}

fn stack(id: &str, name: &str, splays: &[f64], bottom_angle_deg: f64) -> Cluster {
    Cluster {
        id: id.into(),
        name: name.into(),
        schema_version: 1,
        speaker_model_ids: speaker_model_ids_for(splays),
        compartment: Compartment::Stacked,
        joints: joints(splays),
        imposed_tilt: Some(bottom_angle_deg),
        tie_angle: None,
        bumper_model_id: DEFAULT_BUMPER_ID.into(),
        // Un stack est posé : le dessous du bumper est le sol.
        bumper_height: 0.0,
    }
}

/// Grappes à embarquer en seed (brief §8).
pub fn seed_clusters() -> Vec<Cluster> {
    vec![
        flown("droite-12", "Droite 12", &[0.0; 11]),
        flown(
            "banane-douce-12",
            "Banane douce 12",
            &[0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 2.0, 3.0, 4.0, 5.0, 7.0],
        ),
        flown(
            "grosse-banane-12",
            "Grosse banane 12",
            &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0],
        ),
        flown(
            "j-array-14",
            "J-array 14",
            &[
                0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 15.0,
            ],
        ),
        flown(
            "long-splay-8",
            "Long splay 8",
            &[5.0, 6.0, 8.0, 10.0, 12.0, 15.0, 20.0],
        ),
        {
            let splays = [0.0, 0.0, 0.0, 1.0, 1.0, 2.0, 3.0, 5.0, 8.0, 10.0, 12.0];
            Cluster {
                id: "assiette-moins-6".into(),
                name: "Assiette -6".into(),
                schema_version: 1,
                speaker_model_ids: speaker_model_ids_for(&splays),
                compartment: Compartment::Flown,
                joints: joints(&splays),
                imposed_tilt: Some(-6.0),
                tie_angle: None,
                bumper_model_id: DEFAULT_BUMPER_ID.into(),
                bumper_height: DEFAULT_TRIM_HEIGHT_MM,
            }
        },
        stack("stack-classique-3", "Stack classique 3", &[0.0, 20.0], 40.0),
        stack("stack-4-boites", "Stack 4 boites", &[0.0, 0.0, 20.0], 40.0),
        stack("stack-peu-incline", "Stack peu incliné", &[0.0, 10.0], 20.0),
        stack("stack-droit-3", "Stack droit 3", &[0.0, 0.0], 0.0),
    ]
}

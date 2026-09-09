//! Domaine "WST" (Wavefront Sculpture Technology) : les critères de line
//! source d'Urban, Heil & Bauman (AES 5488, §3 à §6.2), calculés pour une
//! ligne donnée. Purement acoustique — aucun lien avec la statique de rigging,
//! qui vit dans `crate::cluster` ; le seul point de contact est le pas entre
//! centres acoustiques, qui se dérive de la géométrie mécanique réelle d'une
//! enceinte (`report::acoustic_step_mm`).
//!
//! `formulas` porte les formules brutes en SI, chacune testée contre les
//! vecteurs publiés ; `report` fait la conversion d'unités et l'assemblage de
//! ce que l'écran affiche.

mod formulas;
mod report;

pub use formulas::{
    arf, arf_for_attenuation, arf_min, axial_loss_db, cca_sagitta_m, curvature_radius_m,
    curved_line_relative_level_db, curved_model_min_splay_rad, first_dip_angle_rad,
    frequency_limit_half_wavelength, grating_lobe_angle_rad, guide_path_delay_m,
    isophase_frequency_limit, line_first_dip_angle_rad, max_frequency_hz, max_splay_rad,
    max_step_m, max_wavefront_deviation_m, min_distance_m, near_field_boundary_fresnel_m,
    near_field_boundary_m, no_near_field_below_hz, side_lobe_attenuation_db, transition_splay_rad,
    wavelength_m, SPEED_OF_SOUND_DEFAULT, SPEED_OF_SOUND_PAPER,
};
pub use report::{
    acoustic_step_mm, front_gap_mm, wst_report, Cca, CcaRow, Criterion1, Criterion2,
    Criterion2Sample, Criterion3, Criterion5, Criterion5AngleLimit, Criterion5Row, CurvatureRow,
    CurvedGuide, CurvedGuideRow, GuideDelaySample, GuideKind, NearField, NearFieldSample,
    WstDerived, WstInputs, WstReport, ANALYSIS_FREQUENCIES_HZ,
};

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
use super::pair::split_over_pair;
use crate::speaker::{CrownRow, JointOffset, RearBar, SpeakerGeometry, SplayRange};
use crate::pull_back::PullBackForce;
use crate::vector::{angle_of, Vec2};
use serde::Serialize;

/// Bras de bielle en dessous duquel la jonction n'a pas de solution. Très en
/// dessous des 629 mm de la géométrie SA303 : cette borne ne se déclenche que
/// sur un perçage aberrant, jamais sur une variation de splay.
pub const MIN_BIELLE_LEVER_MM: f64 = 1.0;

/// Jonction sans solution : la géométrie ne permet pas de résoudre l'équilibre.
/// Remontée plutôt que rendue en NaN — les comparaisons sur NaN étant fausses,
/// un NaN traverserait tous les seuils sans en déclencher aucun.
#[derive(Clone, Debug)]
pub struct JointInconsistency {
    pub joint_index: usize,
    pub reason: String,
}

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
    /// Pull-back actif, vol uniquement.
    pub pull_back: Option<PullBackForce>,
    /// Splay recommandé pour cette paire de modèles, s'il y en a un déclaré.
    pub recommended_splay: Option<SplayRange>,
}

#[derive(Clone, Debug, Serialize)]
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
    ///
    /// **Le trou, pas le chargement.** En vol c'est la goupille de couronne, et
    /// `f_orientation` est bien ce qu'elle voit. En stack c'est l'ancrage, mais
    /// le flanc chargé y porte la **paire** : l'effort réel sur ce trou est
    /// `f_anchor`, pas `f_orientation`, et le verrou en prend autant à côté.
    /// Vérifier ce flanc sur `f_orientation` seul le sous-estime d'un facteur 3.
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

    /// Effort de couronne décomposé dans le repère de la **barre** : axe
    /// `e = (an − bo).normalize()`, dirigé de la couronne vers l'ancrage.
    /// Par flanc, comme tout le reste.
    ///
    /// `bar_axial_n` positif = la barre est comprimée le long de son axe
    /// (l'effort pousse vers l'ancrage) ; négatif = tendue.
    pub bar_axial_n: f64,
    /// Composante transverse, celle qui fait fléchir la barre. C'est elle qui
    /// dimensionne, pas l'axiale.
    pub bar_shear_n: f64,
    /// Moment réduit au barycentre de la paire ancrage/verrou (N·m, par flanc).
    /// Signé, et **exactement** celui qui produit `f_anchor`/`f_latch` : il sort
    /// du même produit vectoriel, pas d'une reconstruction `|V| × bras` qui
    /// perdrait le bras transversal de `up660` et ne coïnciderait donc plus
    /// avec les efforts rendus juste à côté.
    pub bar_moment_at_pair_nm: f64,
    /// Moment de flexion à la **section critique** (N·m, par flanc), c'est-à-dire
    /// à l'**ancrage** : premier pion rencontré depuis la couronne.
    ///
    /// Le moment vaut zéro à la couronne — articulation simple — croît jusqu'à
    /// l'ancrage, puis redescend sous l'effet de la réaction de la paire ; il
    /// est nul au verrou, derrière lequel il ne reste que 14,5 mm de barre où
    /// rien ne s'applique. Les deux pions ont échangé leur rang par rapport à
    /// la géométrie précédente, où le maximum était au verrou.
    pub bar_moment_max_nm: f64,
    /// Abscisse le long de la barre où `bar_moment_max_nm` est atteint, mesurée
    /// depuis l'extrémité couronne.
    pub bar_moment_max_at_mm: f64,
    /// Cotation de la barre de cette jonction. Portée ici pour que la
    /// vérification de section n'ait pas à remonter au modèle d'enceinte : dans
    /// une grappe hétérogène, deux jonctions n'ont pas forcément la même barre.
    pub rear_bar: RearBar,
    /// Les deux efforts que la barre **reçoit**, dans son propre repère et par
    /// flanc : à la couronne, et au pion de verrou. C'est tout ce dont le
    /// balayage de section a besoin — le lui donner ici évite que chaque
    /// appelant les reconstruise, et donc qu'ils divergent.
    pub bar_load_crown: Vec2,
    pub bar_load_latch: Vec2,
    /// Points d'application des deux efforts, et l'ancrage, dans le repère de la
    /// barre — issus de la géométrie **calculée**, pas de la cotation déclarée.
    /// Les deux ne coïncident qu'à la tolérance d'ajustement près, et le
    /// contrôle de raccord du diagramme ne survit pas à ce mélange.
    pub bar_point_crown: Vec2,
    pub bar_point_latch: Vec2,
    pub bar_point_anchor: Vec2,
    /// Abscisse la plus en arrière du bord arrière de barre, repère enceinte :
    /// c'est elle qui doit rester devant la face arrière du caisson. Elle est
    /// au **petit bout**, pas à l'ancrage — le bord est parallèle à l'axe de
    /// barre, qui s'incline vers l'avant en montant.
    pub bar_rear_edge_max_x: f64,
    /// Face arrière du caisson qui porte la paire, pour que la vérification de
    /// barre puisse comparer sans remonter au modèle d'enceinte.
    pub rear_face_x: f64,

    /// Efforts sur les deux goupilles de la paire, repère du flanc chargé, par
    /// flanc. Répartition élastique à raideurs égales : chaque goupille prend la
    /// moitié de la résultante, plus un couple `±P` perpendiculaire à la ligne
    /// ancrage-verrou tel que la somme des moments rende le moment réduit.
    ///
    /// Ce n'est pas une hypothèse à valider mais la solution du problème
    /// élastique quand les deux goupilles ont la même raideur, ce qui est le cas
    /// ici : même diamètre, même épaisseur de barre, même flanc.
    pub f_anchor: Vec2,
    pub f_latch: Vec2,
    pub f_anchor_n: f64,
    pub f_anchor_angle_deg: f64,
    pub f_latch_n: f64,
    pub f_latch_angle_deg: f64,
    /// Positions des trois trous de la barre dans le repère du flanc chargé,
    /// cohérentes entre elles et avec `f_anchor`/`f_latch`/`f_orientation` —
    /// c'est ce qu'il faut pour recouper un moment. À ne pas confondre avec
    /// `loaded_orientation_hole`, qui désigne le trou du flanc chargé et change
    /// donc de nature entre vol et stack.
    pub crown_hole_local: Vec2,
    pub anchor_hole_local: Vec2,
    pub latch_hole_local: Vec2,
    /// Les mêmes en repère global, pour l'`ArrayViewer` : jamais de rotation
    /// côté TypeScript.
    pub anchor_hole_global: Vec2,
    pub latch_hole_global: Vec2,
    /// Efforts sur les deux goupilles de la paire, repère global. Même raison :
    /// le viewer dessine deux flèches à leur point d'application, il ne doit
    /// pas avoir à tourner un vecteur pour ça (brief §1).
    pub f_anchor_global: Vec2,
    pub f_latch_global: Vec2,
    pub residual_n: f64,
    /// Résidu de l'équation de moment, pris ailleurs qu'à la goupille de
    /// couronne (N·mm, effort total et non par flanc). Contrairement à
    /// `residual_n`, il n'est **pas** nul par construction : c'est lui qui
    /// atteste que la résolution est juste.
    pub moment_residual_nmm: f64,

    /// Splay recommandé entre ces deux modèles, s'il y en a un déclaré
    /// (`BelowCompatibility::recommended_splay`), en degrés `[min, max]`.
    pub recommended_splay_range_deg: Option<[f64; 2]>,
    /// `false` uniquement si une recommandation existe et que le splay retenu
    /// en sort. Ce n'est **jamais** une erreur : la jonction reste
    /// mécaniquement valable, elle est seulement signalée comme non optimale
    /// acoustiquement. Sans recommandation déclarée : toujours `true`.
    pub acoustically_optimal: bool,

    /// Vrai sur la jonction 0 d'une grappe suspendue, et là seulement.
    ///
    /// La liaison bumper ↔ premier caisson n'a **pas** été remodélisée : elle
    /// suit encore le schéma antérieur — bras à deux forces entre le trou de
    /// splay 0 et le point d'accroche, plus un pivot fixe au `ht`. Ce n'est pas
    /// la liaison décrite par le modèle en vigueur (bielle bi-goupillée + barre
    /// encastrée sur deux goupilles), et les grandeurs de barre de ce
    /// `JointResult` ne la décrivent donc pas.
    ///
    /// Le drapeau existe pour que ça se voie dans le rapport plutôt que de se
    /// déduire d'une lecture du solveur : une jonction qui ne relève pas du même
    /// modèle que ses voisines ne doit pas se lire sur la même ligne sans
    /// mention.
    pub bumper_model_legacy: bool,
}

impl JointResult {
    /// Les efforts que la barre reçoit, prêts pour `checks::check_bar`. Un seul
    /// endroit où ils se construisent : les reconstruire chez chaque appelant
    /// est le meilleur moyen qu'ils finissent par diverger.
    pub fn bar_loads(&self) -> crate::checks::BarLoads {
        crate::checks::BarLoads {
            crown: self.bar_load_crown,
            crown_at: self.bar_point_crown,
            latch: self.bar_load_latch,
            latch_at: self.bar_point_latch,
            anchor_at: self.bar_point_anchor,
        }
    }

    pub fn mag_orientation(&self) -> f64 {
        self.f_orientation.norm()
    }
    pub fn mag_pivot(&self) -> f64 {
        self.f_pivot.norm()
    }
}

pub fn compute_joint(input: &JointInput) -> Result<JointResult, JointInconsistency> {
    let i = input.joint_index;
    let n = input.speakers.len();
    let si = input.speakers[i];
    let s = input.splays_deg[i];

    // Quincaillerie de la jonction : elle appartient à l'enceinte du haut.
    let geo = &input.chain[i].geo;

    // Les quatre points de la jonction, repère global. `pa`/`pb` sont les deux
    // goupilles de la bielle avant, `bo` la goupille de couronne (articulation
    // simple), `an` l'ancrage de la barre sur le caisson du bas.
    let pa = si.o + geo.hb.rotate(si.phi);
    let pb = si.o + geo.pv_at(s).rotate(si.phi);
    let bo = si.o + geo.crown(s).rotate(si.phi);
    let an = si.o + geo.anchor_at(s).rotate(si.phi);
    let lt = si.o + geo.latch_at(s).rotate(si.phi);

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

    // Le pull-back s'ajoute **sans** `k_dyn`, contrairement aux poids juste
    // au-dessus. Ce n'est pas un oubli : sa tension est calculée en amont
    // (`solver::compute_cluster`) pour tenir une grappe dont le poids est déjà
    // dynamisé — `total_weight_n = masse × g × k_dyn` — donc le facteur y est
    // déjà. Le réappliquer ici le compterait deux fois.
    // Vérifié par `the_pull_back_tension_already_carries_the_dynamic_factor`.
    if let (Compartment::Flown, Some(pull_back)) = (input.compartment, input.pull_back) {
        let last = input.speakers[n - 1];
        let q = last.o + pull_back.point_local.rotate(last.phi);
        rext = rext + pull_back.force;
        mext += (q - bo).cross(pull_back.force);
    }

    // Bielle avant, élément à deux forces : direction imposée par ses deux
    // goupilles, donc une seule inconnue scalaire.
    let u = (pb - pa).normalize();
    let bielle_lever = (pb - bo).cross(u);
    // Sans cette garde, une géométrie où la ligne d'action de la bielle passe
    // par la goupille de couronne rendrait un `lambda` infini, puis des efforts
    // NaN qui traverseraient tout le calcul sans rien déclencher : les
    // comparaisons sur NaN sont fausses, donc aucun seuil ne les arrêterait.
    // 1 mm est très en dessous des 629 mm de la géométrie SA303 — cette borne
    // ne peut se déclencher que sur un perçage aberrant.
    if bielle_lever.abs() < MIN_BIELLE_LEVER_MM {
        return Err(JointInconsistency {
            joint_index: i,
            reason: format!(
                "bras de bielle {bielle_lever:.4} mm au splay {s}° : la ligne d'action \
                 de la bielle passe par la goupille de couronne, la jonction n'a pas \
                 de solution"
            ),
        });
    }
    let lambda = -mext / bielle_lever;
    let f_piv = u * lambda;
    // La goupille de couronne reprend tout le reste : c'est par elle que la
    // barre arrière, encastrée sur le caisson du bas, passe son effort.
    let f_ori = -rext - f_piv;

    // --- Répartition sur la paire ancrage/verrou -----------------------------
    //
    // La barre déverse dans le caisson du bas la résultante `f_pair` et le
    // moment qui l'accompagne, repris par les deux goupilles. À raideurs égales
    // — même diamètre, même épaisseur, même flanc — la solution élastique est
    // la part directe partagée en deux, plus un couple perpendiculaire à la
    // ligne ancrage-verrou.
    //
    // Signe : `P` est construit à partir du moment lui-même par `perp(ab)`,
    // donc `Σ M = M_G` est vrai par construction plutôt que par une convention
    // qu'il faudrait retenir. Le test `pin_pair_reproduces_force_and_moment` le
    // vérifie sur un point quelconque.
    // Effort que la barre applique au caisson du bas. En vol la barre fait
    // partie du corps libre et reçoit `f_ori` à la couronne ; en stack elle est
    // hors du corps libre et reçoit `−f_ori`. Dans les deux cas elle transmet
    // l'opposé à la paire.
    let f_on_bar = match input.compartment {
        Compartment::Flown => f_ori,
        Compartment::Stacked => -f_ori,
    };
    // Ce que les goupilles **subissent** : la barre leur délivre exactement ce
    // qu'elle a reçu à la couronne, moment compris.
    let (f_anchor_g, f_latch_g, m_g) = split_over_pair(an, lt, bo, f_on_bar);
    // Exactement le moment qui produit le couple ci-dessus — pas une
    // reconstruction `|V| × bras`, qui perdrait le bras transversal de `up660`
    // et ne coïnciderait donc plus avec les efforts réellement rendus.
    let bar_moment_at_pair_nm = m_g * input.share_per_flank / 1000.0;

    let ti = match input.compartment {
        Compartment::Flown => i,
        Compartment::Stacked => i + 1,
    };
    // Trous exprimés dans le repère du flanc chargé : c'est donc la géométrie
    // de CETTE enceinte-là (celle du haut en vol, celle du bas en stack).
    let loaded = &input.chain[ti];
    let rt_phi = input.speakers[ti].phi;
    let sg = -input.share_per_flank;

    let f_orientation = (f_ori * sg).rotate_transpose(rt_phi);
    let f_pivot = (f_piv * sg).rotate_transpose(rt_phi);
    // Les goupilles de la paire subissent déjà l'action de la barre : c'est
    // `+share` et non `sg = −share`, sans quoi elles sortiraient à l'envers des
    // deux autres.
    let to_local = |p: Vec2| (p - input.speakers[ti].o).rotate_transpose(rt_phi);
    let f_anchor = (f_anchor_g * input.share_per_flank).rotate_transpose(rt_phi);
    let f_latch = (f_latch_g * input.share_per_flank).rotate_transpose(rt_phi);
    let gravity_local = Vec2::new(0.0, -1.0).rotate_transpose(rt_phi);
    let ext_local = (rext * input.share_per_flank).rotate_transpose(rt_phi);
    // Résidu de FORCE : nul par construction, puisque `f_ori` est posé à
    // `−rext − f_piv`. Conservé parce qu'il attrape une faute de signe ou de
    // repère dans les rotations, mais il ne teste pas la résolution.
    let residual = (f_orientation + f_pivot - ext_local).norm();
    // Résidu de MOMENT, lui, autour d'un point **autre** que la goupille de
    // couronne. À `bo` il serait nul par construction lui aussi — c'est là
    // qu'on a annulé l'inconnue vectorielle. Pris en `pa` (goupille haute de
    // bielle) et au centre de masse, il vérifie réellement que `lambda` sort
    // juste : une erreur de bras ou de signe s'y voit immédiatement.
    let moment_residual_nmm = [pa, cm]
        .into_iter()
        .map(|o| ((bo - o).cross(f_ori) + (pb - o).cross(f_piv) + (cm - o).cross(rext)).abs())
        .fold(0.0_f64, f64::max);

    // Rangée de couronne : déclarée par l'enceinte, pas déduite du splay.
    let crown_row = loaded.geo.crown_row_at(s);

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
    let constrained_crown_row = constrained_crown_splay.map(|cs| loaded.geo.crown_row_at(cs));
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
    // --- La barre arrière, dans son propre repère ---------------------------
    //
    // Le repère de barre : abscisse depuis le petit bout (côté verrou), latéral
    // positif vers l'avant du caisson. L'axe est la droite ancrage -> couronne
    // extérieure ; verrou, ancrage et `up680` y sont tous à latéral nul, ce qui
    // est vérifié au chargement par `check_rear_bar`.
    //
    // **`up660` n'est pas sur l'axe.** Sur les splays impairs, l'effort de
    // couronne s'applique 19,4 mm en travers. Le moment ne peut donc pas
    // s'écrire `|V| × abscisse` : il faut le produit vectoriel complet, sans
    // quoi le bras transversal disparaîtrait et avec lui la part de moment
    // qu'il introduit.
    let bar = &input.chain[i].rear_bar;
    // Direction d'abscisse croissante, du verrou vers la couronne. Prise sur
    // les deux trous de la paire, qui sont sur l'axe par construction.
    let e_axis = (an - lt).normalize();
    // Normale « vers l'avant », même construction qu'en géométrie : tourner
    // l'axe d'un quart de tour direct. Nommée plutôt que laissée à un ordre de
    // produit vectoriel — l'inverser mettrait `up660` du mauvais côté.
    let e_front = Vec2::new(-e_axis.y, e_axis.x);

    // Effort vu par la barre, exprimé dans son repère.
    let f_bar = Vec2::new(f_on_bar.dot(e_axis), f_on_bar.dot(e_front));
    let p_crown = bar.crown_hole(crown_row).as_vec();
    let p_anchor = bar.holes.anchor.as_vec();

    let sf = input.share_per_flank;
    // Axial compté positif en compression, donc le long de l'axe **descendant**
    // vers la paire : c'est le sens dans lequel la couronne pousse la barre.
    let bar_axial_n = -f_bar.x * sf;
    let bar_shear_n = f_bar.y * sf;

    // Les deux efforts que la barre reçoit, dans son repère et par flanc. Le
    // pion de verrou lui applique l'opposé de ce qu'il subit.
    let to_bar = |v: Vec2| Vec2::new(v.dot(e_axis), v.dot(e_front));
    let bar_load_crown = to_bar(f_on_bar) * sf;
    let bar_load_latch = to_bar(-f_latch_g) * sf;
    // Les trois points, mesurés depuis le petit bout de la barre le long de son
    // axe réel. L'origine est posée à l'abscisse déclarée du verrou : c'est elle
    // qui cale le profil de largeur sur la pièce.
    let bar_origin = lt - e_axis * bar.holes.latch.along();
    let to_bar_point = |p: Vec2| {
        let v = p - bar_origin;
        Vec2::new(v.dot(e_axis), v.dot(e_front))
    };
    let bar_point_crown = to_bar_point(bo);
    let bar_point_latch = to_bar_point(lt);
    let bar_point_anchor = to_bar_point(an);
    // Point le plus en arrière du bord arrière de barre. Le bord est parallèle
    // à l'axe et décalé de `rear_edge_offset` vers l'arrière ; son abscisse
    // maximale est donc au bout le plus bas de la barre, l'abscisse 0.
    let bar_rear_edge_max_x = {
        let rear_normal = Vec2::new(e_axis.y, -e_axis.x);
        let origin = lt - e_axis * bar.holes.latch.along();
        let tip = origin + rear_normal * bar.rear_edge_offset;
        let far = origin + e_axis * bar.length + rear_normal * bar.rear_edge_offset;
        // Exprimé dans le repère du caisson qui porte la paire, pour se
        // comparer à sa face arrière. Le caisson du bas est calé par son `ht`
        // sur la goupille basse de bielle **et** tourné du splay entier : sans
        // cette rotation, l'erreur atteint 26 mm à 5° sur la longueur de barre.
        let to_pair_frame = |p: Vec2| {
            let in_upper = (p - input.speakers[i].o).rotate_transpose(input.speakers[i].phi);
            (in_upper - geo.pv_at(s)).rotate(-s.to_radians()) + geo.ht
        };
        to_pair_frame(tip).x.max(to_pair_frame(far).x)
    };

    // Moment exact en une section, vu depuis le côté couronne : entre la
    // couronne et l'ancrage, la barre ne voit que cet effort-là.
    let moment_from_crown = |p: Vec2| (p - p_crown).cross(f_bar) * sf;

    // Section critique : l'**ancrage**, premier pion rencontré depuis la
    // couronne. Au-delà, la réaction de la paire fait redescendre le moment,
    // qui est nul au verrou — il ne reste que 14,5 mm de barre derrière lui, et
    // rien ne s'y applique. Les deux pions ont échangé leur rang par rapport à
    // la géométrie précédente : c'était le verrou, c'est l'ancrage.
    let bar_moment_max_nm = moment_from_crown(p_anchor).abs() / 1000.0;
    let bar_moment_max_at_mm = p_anchor.x;

    // Traction de la barre : signe de sa composante axiale.
    let traction = match input.compartment {
        Compartment::Stacked => bar_axial_n <= 0.0,
        Compartment::Flown => bar_axial_n >= 0.0,
    };

    let free_body_count = match input.compartment {
        Compartment::Flown => n - i - 1,
        Compartment::Stacked => i + 1,
    };

    Ok(JointResult {
        joint_index: i,
        compartment: input.compartment,
        free_body_count,
        loaded_flank: ti,
        inclination_deg: rt_phi.to_degrees(),
        splay_deg: s,
        row: crown_row,
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
        bar_axial_n,
        bar_shear_n,
        bar_moment_at_pair_nm,
        bar_moment_max_nm,
        bar_moment_max_at_mm,
        rear_bar: bar.clone(),
        bar_load_crown,
        bar_load_latch,
        bar_point_crown,
        bar_point_latch,
        bar_point_anchor,
        bar_rear_edge_max_x,
        rear_face_x: input.chain[i].rear_face_x,
        f_anchor,
        f_latch,
        f_anchor_n: f_anchor.norm(),
        f_anchor_angle_deg: angle_of(f_anchor),
        f_latch_n: f_latch.norm(),
        f_latch_angle_deg: angle_of(f_latch),
        crown_hole_local: to_local(bo),
        anchor_hole_local: to_local(an),
        latch_hole_local: to_local(lt),
        anchor_hole_global: an,
        latch_hole_global: lt,
        f_anchor_global: f_anchor_g * input.share_per_flank,
        f_latch_global: f_latch_g * input.share_per_flank,
        residual_n: residual,
        moment_residual_nmm,
        recommended_splay_range_deg: input.recommended_splay.map(|r| [r.min_deg, r.max_deg]),
        acoustically_optimal: input.recommended_splay.is_none_or(|r| r.contains(s)),
        bumper_model_legacy: matches!(input.compartment, Compartment::Flown) && i == 0,
    })
}

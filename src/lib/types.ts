// Miroir TypeScript des types serde de `sa303-core` (camelCase). Ce fichier ne
// contient aucune formule : le calcul reste entièrement côté Rust (brief §1).

export type Compartment = "flown" | "stacked";
export type CrownRow = "ext" | "int";

export interface Vec2 {
  x: number;
  y: number;
}

export interface Hinge {
  x: number;
  y: number;
  /** Distance verticale centre à centre au splay 0 : `2*y + entraxe de bielle`. */
  jointSeparation: number;
  /** Recul du trou de charnière depuis la face avant (`e_perp`, EN 1993-1-8 t.3.9). */
  edgePerp: number;
}

export interface RearBar {
  thickness: number;
  length: number;
  wideWidth: number;
  /** Longueur de la section large, depuis l'extrémité couronne. C'est ce
   * bout-là qui est large. */
  wideLength: number;
  narrowWidth: number;
  holeDiameter: number;
  /** Deux trous de couronne : l'entraxe couronne-ancrage n'est pas le même sur
   * les deux couronnes. */
  crownHoleOuterAt: number;
  crownHoleInnerAt: number;
  latchHoleAt: number;
  anchorHoleAt: number;
  yieldStrength: number;
  ultimateStrength: number;
}

export interface Crown {
  radius: number;
  delta: number;
  anchorAngle: number;
  /** Second point de fixation de la barre arrière : c'est lui qui l'encastre
   * sur le caisson du bas, donc lui transmet un moment en plus d'une force. */
  latchAngle: number;
  splay0Angle: number;
}

/**
 * Nature du front rayonné par le guide. C'est elle qui décide des critères WST
 * applicables : le critère 5 ne juge que des fronts plans anglés.
 */
export type WaveguideFront = "isophase" | "constantCurvature";

export interface SpeakerAcoustics {
  fs: number;
  directivityHorizontal: number;
  /** Ouverture verticale d'une caisse seule, degrés. Pour un guide à courbure
   * constante, c'est le secteur θ rayonné, donc le splay maximal exploitable.
   * 0 pour un front isophase. */
  directivityVertical: number;
  wgFront: WaveguideFront;
  /** Hauteur de bouche du guide, mm — le `D` des critères WST.
   * 0 = non renseignée. */
  wgOutputHeight: number;
  /** Niveau du guide à la moitié de son secteur (dB, négatif). −6 dB donne un
   * raccord plat entre deux caisses voisines. */
  wgLevelAtHalfCoverageDb: number;
}

/** Plage de splay recommandée pour une jonction, en degrés. Hors plage, la
 * jonction reste mécaniquement valable : elle est seulement signalée comme non
 * optimale acoustiquement. */
export interface SplayRange {
  minDeg: number;
  maxDeg: number;
}

/** Ce qui peut être accroché directement SOUS une enceinte donnée. La relation
 * est déclarée dans un seul sens (vers le bas) : « ce qui peut aller
 * au-dessus » se déduit en balayant les modèles qui déclarent celui-ci. */
export interface BelowCompatibility {
  speakerModelId: string;
  flown: boolean;
  stacked: boolean;
  recommendedSplay?: SplayRange | null;
}

// Mécanique aplatie au premier niveau (`#[serde(flatten)]` côté Rust). Chaque
// modèle est une enceinte à part entière (SA303-ISOPHASE, SA303-CCA, renfort
// de grave, ...) : ce qui les relie, c'est `compatibleBelow`.
export interface SpeakerModel {
  id: string;
  name: string;
  schemaVersion: number;
  depth: number;
  height: number;
  totalVerticalAngle: number;
  massKg: number;
  cg: [number, number];
  hinge: Hinge;
  crown: Crown;
  splayGrid: number[];
  frameHoleSplay: number;
  rearBar: RearBar;
  acoustics: SpeakerAcoustics;
  compatibleBelow: BelowCompatibility[];
}

export interface JointSetting {
  splay: number;
}

/** Identifiants des composants livrés avec le logiciel. Immuables dans
 * l'application : seule une mise à jour les modifie. */
export interface BuiltinIds {
  speakers: string[];
  bumpers: string[];
  bumperBars: string[];
  clusters: string[];
}

export interface Cluster {
  id: string;
  name: string;
  schemaVersion: number;
  /** Une enceinte par position, du haut vers le bas : une grappe est
   * hétérogène. Compte exactement une entrée de plus que `joints`. */
  speakerModelIds: string[];
  compartment: Compartment;
  /** Du haut vers le bas : exactement `speakerModelIds.length - 1`. */
  joints: JointSetting[];
  imposedTilt?: number | null;
  /** Direction de traction choisie pour la tirette automatique (degrés,
   * convention §2), si le bumper et sa barre ne suffisent plus. `null` tant
   * que l'utilisateur n'a pas encore choisi : le solveur suggère alors la
   * direction qui minimise la tension, affichée mais jamais imposée. */
  tieAngle?: number | null;
  /** Bumper utilisé pour dériver le point d'accroche en vol ou comme support
   * en stack. Obligatoire : il n'existe pas de repli manuel. */
  bumperModelId: string;
  /** Altitude du **dessous** du bumper au-dessus du sol, mm. En stack, 0 = posé
   * au sol ; en vol, c'est la hauteur d'accroche. N'entre dans aucun calcul
   * d'effort : elle situe l'ensemble dans l'espace. */
  bumperHeight: number;
}

export interface BumperCompatibility {
  speakerModelId: string;
  flown: boolean;
  stacked: boolean;
}

export interface BumperModel {
  id: string;
  name: string;
  schemaVersion: number;
  depth: number;
  height: number;
  shackleHeightAboveBumper: number;
  /** Décalage max avant qu'une SA303-BUMPER-BAR ne soit nécessaire, mm (déf. depth/2). */
  maxDirectDeportMm: number;
  compatibleSpeakers: BumperCompatibility[];
}

export interface BumperBarCompatibility {
  bumperModelId: string;
}

export interface BumperBarModel {
  id: string;
  name: string;
  schemaVersion: number;
  /** Portée max de déport depuis le centre du bumper, mm : au-delà, une tirette prend le relais. */
  maxDeportMm: number;
  compatibleBumpers: BumperBarCompatibility[];
}

export interface PinSpec {
  diameter: number;
  ultimate: number;
  netSection: number;
}

export interface PlateSpec {
  flankThickness: number;
  barThickness: number;
  ultimate: number;
}

export interface AxisMapping {
  toolX: string;
  toolY: string;
}

export interface Settings {
  safetyFactor: number;
  dynamicFactor: number;
  gravity: number;
  sharePerFlank: number;
  pin: PinSpec;
  plate: PlateSpec;
  axisMapping: AxisMapping;
}

export interface SpeakerInstance {
  o: Vec2;
  phi: number;
  /** CG global de cette enceinte — jamais à recalculer côté front. */
  cg: Vec2;
  /** Silhouette (trapèze) en repère enceinte : propre à CETTE position, deux
   * enceintes d'une même grappe pouvant être de modèles différents. */
  outline: [Vec2, Vec2, Vec2, Vec2];
}

export interface JointResult {
  jointIndex: number;
  compartment: Compartment;
  freeBodyCount: number;
  loadedFlank: number;
  inclinationDeg: number;
  splayDeg: number;
  row: CrownRow;
  crownRadius: number;
  /** Bras couronne-ancrage, recoupement de perçage : plus utilisé par la statique. */
  leverMm: number;
  /** Bras de la bielle avant autour de la goupille de couronne : celui qui
   * résout réellement la jonction. */
  bielleLeverMm: number;
  /** Rotation de la bielle avant : exactement la moitié du splay. */
  bielleRotationDeg: number;
  /** Écartement des coins avant. C'est `verticalMm` qu'on affiche comme
   * espacement entre caissons — `frontMm` reste sous 0.05 mm en line source. */
  offset: JointOffset;
  loadedOrientationHole: Vec2;
  loadedPivotHole: Vec2;
  constrainedHingeHole: Vec2;
  constrainedCrownSplay: number | null;
  constrainedCrownHole: Vec2 | null;
  constrainedCrownRow: CrownRow | null;
  constrainedCrownRadius: number | null;
  constrainedIsFrame: boolean;
  fOrientation: Vec2;
  fPivot: Vec2;
  gravityLocal: Vec2;
  /** Intensité (N) et direction (convention §2, degrés) pré-calculées : jamais
   * de Math.atan2 côté front. */
  fOrientationN: number;
  fOrientationAngleDeg: number;
  fPivotN: number;
  fPivotAngleDeg: number;
  /** Mêmes trous/efforts en repère global — à utiliser tels quels dans l'ArrayViewer. */
  loadedOrientationHoleGlobal: Vec2;
  loadedPivotHoleGlobal: Vec2;
  fOrientationGlobal: Vec2;
  fPivotGlobal: Vec2;
  traction: boolean;
  hingeReversed: boolean;
  /** Décomposition de l'effort de couronne dans le repère de la barre, par
   * flanc. Axial positif = barre comprimée. */
  barAxialN: number;
  barShearN: number;
  /** Moment réduit au barycentre de la paire ancrage/verrou, N·m par flanc. */
  barMomentAtPairNm: number;
  /** Moment de flexion maximal dans la barre, N·m par flanc. Nul à la couronne
   * (articulation), maximal au premier trou de la paire. */
  barMomentMaxNm: number;
  barMomentMaxAtMm: number;
  rearBar: RearBar;
  /** Efforts sur les deux goupilles de la paire, repère du flanc chargé, par
   * flanc. Répartition élastique à raideurs égales. */
  fAnchor: Vec2;
  fLatch: Vec2;
  fAnchorN: number;
  fAnchorAngleDeg: number;
  fLatchN: number;
  fLatchAngleDeg: number;
  /** Les trois trous de barre dans le repère du flanc chargé, cohérents entre
   * eux et avec les efforts — de quoi recouper un moment. */
  crownHoleLocal: Vec2;
  anchorHoleLocal: Vec2;
  latchHoleLocal: Vec2;
  anchorHoleGlobal: Vec2;
  latchHoleGlobal: Vec2;
  /** Efforts sur les deux goupilles de la paire, repère global — le viewer
   * dessine deux flèches à leur point d'application, il n'a pas à tourner un
   * vecteur pour ça. */
  fAnchorGlobal: Vec2;
  fLatchGlobal: Vec2;
  residualN: number;
  /** Résidu de l'équation de moment, pris ailleurs qu'à la goupille de
   * couronne. Contrairement à `residualN`, il n'est pas nul par construction. */
  momentResidualNmm: number;
  /** Splay recommandé entre les deux modèles de cette jonction, s'il y en a un
   * déclaré, en degrés `[min, max]`. */
  recommendedSplayRangeDeg: [number, number] | null;
  /** `false` uniquement si une recommandation existe et que le splay en sort.
   * Jamais une erreur : la jonction reste mécaniquement valable. */
  acousticallyOptimal: boolean;
  /** Vrai sur la jonction 0 d'une grappe suspendue, et là seulement : la
   * liaison bumper ↔ premier caisson suit encore le schéma antérieur (bras à
   * deux forces + pivot fixe), pas le modèle bielle + barre encastrée. Les
   * grandeurs de barre ne la décrivent donc pas. */
  bumperModelLegacy: boolean;
}

/** Altitudes au-dessus du sol, mm. Purement descriptif : aucun effort n'en
 * dépend. */
export interface Elevation {
  /** À ajouter à un `y` du repère global pour obtenir une altitude — de quoi
   * situer n'importe quel point, par exemple sous le curseur. */
  offsetMm: number;
  /** Dessous du bumper : exactement la valeur saisie sur la grappe. */
  bumperBottomMm: number;
  /** Point le plus bas de l'ensemble, bumper compris. */
  lowestPointMm: number;
  highestPointMm: number;
  /** Vol uniquement : altitude du point de levage. */
  pickupMm: number | null;
  /** Altitude du dessous de chaque enceinte, même ordre que `speakers` : la
   * cote qu'un rigger lit au mètre. Calculée côté Rust — c'est le coin le plus
   * bas de la silhouette une fois tournée. */
  speakerBottomMm: number[];
}

export interface ClusterResult {
  speakers: SpeakerInstance[];
  phiInitial: number;
  /** Angle qu'adopterait la grappe en pendaison libre (accroche centrée sur
   * le bumper), toujours calculé en vol, indépendamment d'une éventuelle
   * assiette imposée — référence de comparaison, jamais remplacée par
   * `phiInitial`. `null` en stack. */
  phiFreeHang: number | null;
  tieTensionN: number;
  joints: JointResult[];
  /** CG de l'ensemble, repère global : barycentre pondéré par la masse de
   * chaque enceinte. */
  cg: Vec2;
  /** Masse totale réellement montée, kg — jamais une masse unitaire à
   * multiplier côté front. */
  totalMassKg: number;
  /** Nom du modèle monté à chaque position, même ordre que `speakers`. */
  speakerNames: string[];
  /** Point de levage, repère global. Vol uniquement. */
  pickupGlobal: Vec2 | null;
  /** Point d'accroche de la tirette sur l'enceinte du bas, repère global. */
  tiePointGlobal: Vec2 | null;
  /** Direction de traction de la tirette, repère global. */
  tieDirectionGlobal: Vec2 | null;
  /** Même direction, en degrés (convention §2) : angle réel auquel ancrer la
   * tirette (choisi par l'utilisateur, ou suggéré par le solveur tant qu'il
   * n'a pas encore choisi — voir `BumperView.tieAngleRangeDeg`). */
  tieDirectionAngleDeg: number | null;
  /** Position de l'ensemble dans l'espace, dérivée de `Cluster.bumperHeight`. */
  elevation: Elevation;
  /** Bumper attaché à l'enceinte de référence. Toujours présent : un bumper
   * est obligatoire. */
  bumperView: BumperView;
}

export interface BumperView {
  outlineGlobal: [Vec2, Vec2, Vec2, Vec2];
  /** Vol uniquement : point d'accroche effectif (sur le bumper, ou sur la
   * barre de déport si `barDeportMm != 0`). Toujours renseigné en vol. */
  pickupGlobal: Vec2 | null;
  /** Départ de la barre de déport sur le bord de la zone de fixation directe.
   * Absent si aucune barre n'est nécessaire. */
  bumperBarStartGlobal: Vec2 | null;
  /** Position de l'accroche par rapport au **centre du bumper** (mm signés,
   * positif vers l'arrière) : la cote que le rigger reporte. Non nulle dès que
   * l'accroche n'est pas centrée, y compris quand elle reste sur le bumper. */
  pickupOffsetMm: number | null;
  /** Ce que la **barre** porte : dépassement signé au-delà de
   * `maxDirectDeportMm`, donc 0 tant que l'accroche tombe sur le bumper. Ne
   * décrit pas où est l'accroche, mais s'il faut une barre et de combien. */
  barDeportMm: number | null;
  /** Au-delà de la portée de la barre (`BumperBarModel.maxDeportMm`), elle ne suffit plus : une tirette est
   * automatiquement mise en place (voir `tieTensionN`/`tiePointGlobal` sur
   * ClusterResult). */
  bumperBarExceeded: boolean;
  /** Plage de directions de traction physiquement valables (degrés,
   * convention §2), présente seulement si `bumperBarExceeded`. Le point d'ancrage
   * réel dépend du terrain : à choisir dedans, ce n'est pas à l'algorithme
   * de décider seul. */
  tieAngleRangeDeg: [number, number] | null;
  /** Efforts transmis par le bumper à l'enceinte de référence, repère de
   * cette enceinte, par flanc. Vol uniquement. */
  orientationForceN: number | null;
  orientationAngleDeg: number | null;
  pivotForceN: number | null;
  pivotAngleDeg: number | null;
  /** Les deux mêmes en repère global, avec leur point d'application sur
   * l'enceinte de référence : le viewer les dessine tels quels. */
  orientationPointGlobal: Vec2 | null;
  pivotPointGlobal: Vec2 | null;
  orientationForceGlobal: Vec2 | null;
  pivotForceGlobal: Vec2 | null;
  /** Charge que le bumper reprend en entier : la manille en vol, la réaction du
   * sol en stack. **Pas** par flanc — une manille n'est pas doublée. */
  supportForceN: number;
  supportForceGlobal: Vec2;
  supportPointGlobal: Vec2;
  supportAngleDeg: number;
}

export interface JointOffset {
  frontMm: number;
  verticalMm: number;
}

export interface CrownHoleReport {
  splayDeg: number;
  row: CrownRow;
  radius: number;
  position: Vec2;
  /** Goupille basse de bielle à ce splay : le centre depuis lequel ce trou est
   * percé. Il bouge d'un cran à l'autre — les trous ne sont pas sur un arc. */
  pv: Vec2;
  offset: JointOffset;
  leverMm: number;
  leverCheckMm: number;
  discrepancyMm: number;
}

export interface SpeakerGeometryReport {
  ha: number;
  ht: Vec2;
  hb: Vec2;
  /** Goupille basse de bielle au splay 0. Ce n'est pas un pivot fixe. */
  pv0: Vec2;
  anchorLocal: Vec2;
  latchLocal: Vec2;
  frontEdge: Vec2;
  bielleEntraxe: number;
  holes: CrownHoleReport[];
  /** Silhouette de l'enceinte (trapèze), repère enceinte — pour l'ArrayViewer. */
  outline: [Vec2, Vec2, Vec2, Vec2];
}

export interface LoadCaseReport {
  clusterName: string;
  jointNumber: number;
  labels: string[];
  duplicate: boolean;
  result: JointResult;
  /** Sandwich à la goupille de couronne. */
  utilizationOrientation: number;
  /** Sandwich aux goupilles de bielle. */
  utilizationPivot: number;
  /** Sandwich aux deux goupilles de la paire. Elles ne culminent pas au même
   * endroit : chacune porte la part directe plus ou moins le couple. */
  utilizationAnchor: number;
  utilizationLatch: number;
  /** Flexion composée de la barre arrière. */
  utilizationBar: number;
  barCheck: BarCheck;
  /** Le pire des cinq chemins. C'est lui qui dit si la jonction passe :
   * `utilizationOrientation` seul sous-estime la paire d'un facteur 3 et ignore
   * complètement la flexion. */
  utilizationWorst: number;
}

export interface CompartmentReport {
  blockA: LoadCaseReport[];
  blockB: LoadCaseReport[];
  /** Trous percés qu'aucune grappe de ce compartiment n'exploite : l'enveloppe
   * ne dimensionne rien pour ces angles. Signalés plutôt que tus. */
  uncoveredSplaysDeg: number[];
}

export interface ImpossibleClusterReport {
  clusterName: string;
  reason: string;
}

export interface AggregateReport {
  flown: CompartmentReport;
  stacked: CompartmentReport;
  /** Grappes exclues des blocs car leur configuration est physiquement impossible. */
  impossibleClusters: ImpossibleClusterReport[];
}

// --- Critères WST (Urban, Heil & Bauman, AES 5488) --------------------------
// Tout est calculé côté Rust : ces types ne servent qu'à mettre en page.

/**
 * Nature du front rayonné par un élément. C'est elle qui décide des critères
 * applicables : angler deux fronts **plans** ouvre entre eux une zone sans
 * énergie (critère 5 du papier), alors qu'un guide rayonnant déjà un secteur
 * juxtapose le sien à celui de son voisin — il n'y a rien à refermer.
 */
export type WstGuideKind =
  | { kind: "isophase" }
  | { kind: "curved"; coverageDeg: number; levelAtHalfSplayDb: number };

export interface WstInputs {
  speedOfSound: number;
  boxHeightMm: number;
  gapMm: number;
  radiatingHeightMm: number;
  speakerCount: number;
  splaysDeg: number[];
  distancesM: number[];
  fMaxHz: number;
  guide: WstGuideKind;
}

export interface WstDerived {
  stepMm: number;
  gapMm: number;
  arf: number;
  lineHeightM: number;
  /** Bouche réellement utilisée : celle de l'enceinte si elle en déclare une. */
  radiatingHeightMm: number;
  /** Guide réellement utilisé — l'enceinte l'emporte sur la saisie. */
  guide: WstGuideKind;
  /** Pas dérivé de la géométrie d'une enceinte (il varie alors avec l'angle). */
  stepFromSpeaker: boolean;
}

export interface WstCriterion1 {
  arf: number;
  arfMin: number;
  satisfied: boolean;
  /** `null` à ARF ≥ 1 : ligne continue, pas de lobe de réseau. */
  sideLobeAttenuationDb: number | null;
  axialLossDb: number;
}

export interface WstCriterion2Sample {
  frequencyHz: number;
  satisfied: boolean;
  gratingLobeDeg: number | null;
  firstDipDeg: number | null;
}

export interface WstCriterion2 {
  frequencyLimitHz: number;
  samples: WstCriterion2Sample[];
}

export interface WstCriterion3 {
  fMaxHz: number;
  maxDeviationMm: number;
}

export interface WstNearFieldSample {
  frequencyHz: number;
  boundaryM: number | null;
  boundaryFresnelM: number;
  firstDipDeg: number | null;
}

export interface WstNearField {
  noNearFieldBelowHz: number;
  samples: WstNearFieldSample[];
}

export interface WstCriterion5Row {
  splayDeg: number;
  stepMm: number;
  /** Jour en façade à cet angle. Variable d'une ligne à l'autre dès qu'une
   * enceinte est sélectionnée : la charnière est en retrait de la face, donc
   * incliner fait bâiller la façade, et le pas s'ouvre d'autant. */
  gapMm: number;
  arf: number;
  /** Même ordre que `WstInputs.distancesM`. */
  fMaxByDistanceHz: (number | null)[];
}

export interface WstCriterion5AngleLimit {
  frequencyHz: number;
  /** `null` : aucun angle admissible à cette fréquence pour cette distance. */
  maxSplayDegByDistance: (number | null)[];
}

export interface WstCriterion5 {
  rows: WstCriterion5Row[];
  angleLimits: WstCriterion5AngleLimit[];
  maxStepMm: number | null;
  closestDistanceM: number | null;
}

export interface WstCurvatureRow {
  splayDeg: number;
  radiusM: number | null;
  angleDistanceProduct: number | null;
  relativeLevelDb: number | null;
  curvedModelMinSplayDeg: number;
}

export interface WstCcaRow {
  splayDeg: number;
  radiusM: number | null;
  sagittaMm: number | null;
  /**
   * Au-dessus de cette fréquence, la flèche d'une corde plate sur l'arc dépasse
   * λ/4 : la courbure du front doit être juste. En dessous, elle est
   * indifférente.
   */
  curvatureMattersAboveHz: number | null;
}

export interface WstGuideDelaySample {
  yMm: number;
  delayMm: number;
}

export interface WstCca {
  rows: WstCcaRow[];
}

export interface WstCurvedGuideRow {
  splayDeg: number;
  /** `θ_guide ≥ α` : les secteurs voisins se juxtaposent sans laisser de trou. */
  covered: boolean;
  targetRadiusM: number | null;
  curvatureMattersAboveHz: number | null;
}

export interface WstCurvedGuide {
  coverageDeg: number;
  maxSplayDeg: number;
  levelAtHalfSplayDb: number;
  /** Niveau au raccord : deux secteurs voisins s'y somment, soit +6 dB. */
  spliceLevelDb: number;
  rows: WstCurvedGuideRow[];
  transitionSplayDeg: number;
  guideDelayProfile: WstGuideDelaySample[];
}

export interface WstReport {
  derived: WstDerived;
  criterion1: WstCriterion1;
  criterion2: WstCriterion2;
  criterion3: WstCriterion3;
  nearField: WstNearField;
  /** Renseigné uniquement en front plan : c'est le seul cas que juge le papier. */
  criterion5: WstCriterion5 | null;
  curvature: WstCurvatureRow[];
  cca: WstCca;
  /** Renseigné uniquement pour un guide à front courbé. */
  curvedGuide: WstCurvedGuide | null;
}

/* ---------------------------------------------------------------------------
 * Export d'audit (brief : valider le calcul avec un tiers).
 *
 * Document autoportant : les définitions des pièces d'abord, les résultats
 * ensuite. Un relecteur qui reçoit ce fichier n'a ni le logiciel, ni le
 * catalogue, ni les réglages — tout ce qu'il faut pour refaire le calcul doit
 * donc être dedans.
 * ------------------------------------------------------------------------- */

/** Une section vérifiée de la barre arrière. */
export interface BarSectionCheck {
  location: string;
  atMm: number;
  widthMm: number;
  drilled: boolean;
  momentNmm: number;
  axialN: number;
  stressMpa: number;
  /** Rapport à R_m/sf — le critère. */
  utilization: number;
  /** Rapport à f_y, indicatif : ce n'est pas le critère retenu. */
  yieldRatio: number;
}

export interface BarCheck {
  sections: BarSectionCheck[];
  /** Index de la section la plus sollicitée dans `sections`. */
  worst: number;
}

/** Les cinq chemins de charge d'une jonction, chacun rapporté à son admissible. */
export interface JointChecks {
  utilizationCrown: number;
  utilizationBielle: number;
  utilizationAnchor: number;
  utilizationLatch: number;
  utilizationBar: number;
  utilizationWorst: number;
  /** Nom du chemin qui gouverne : l'information qui dit quoi renforcer. */
  governingPath: string;
  bar: BarCheck;
}

export interface ClusterExport {
  definition: Cluster;
  result: ClusterResult;
  /** Un par jonction, même ordre que `result.joints`. */
  jointChecks: JointChecks[];
  utilizationWorst: number;
}

export interface ImpossibleClusterExport {
  id: string;
  name: string;
  reason: string;
}

export interface AuditExportDefinitions {
  speakers: SpeakerModel[];
  bumpers: BumperModel[];
  bumperBars: BumperBarModel[];
}

export interface AuditExport {
  schemaVersion: number;
  generatedAt: string;
  /** Réglages sous lesquels tout le reste a été calculé. Sans eux, aucun taux
   * du document n'est reproductible. */
  settings: Settings;
  definitions: AuditExportDefinitions;
  clusters: ClusterExport[];
  /** Grappes écartées, avec leur raison. Jamais omises en silence. */
  impossible: ImpossibleClusterExport[];
}

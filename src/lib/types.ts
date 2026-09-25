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
  /** Splays percés sur la rangée **intérieure**, en retrait de `delta`. Tout
   * splay absent de cette liste est sur la rangée extérieure.
   *
   * Déclaré, jamais déduit : la règle précédente lisait la parité du splay
   * arrondi, ce qui rangeait un 10,5° par un arrondi que personne n'avait
   * choisi. Le perçage est une donnée du plan de l'enceinte. */
  innerSplays: number[];
}

/**
 * Nature du front rayonné par le guide. C'est elle qui décide des critères WST
 * applicables : le critère 5 ne juge que des fronts plans anglés.
 */
export type WaveguideFront = "isophase" | "constantCurvature";

/**
 * Ce qu'on a réellement relevé du guide, par identification d'un modèle
 * d'ouverture sur des polaires simulées ou mesurées.
 *
 * Front **plan** : `acousticMouthHeightMm` est le `D` équivalent qui pilote le
 * critère 5. Front **courbé** : les deux premiers champs sont corrélés par
 * l'ajustement, seul leur rapport `D/R` — le secteur — est déterminé, et le `D`
 * identifié n'est alors PAS une hauteur de bouche.
 */
export interface GuideMeasurement {
  acousticMouthHeightMm: number;
  /** Rayon du front identifié, m. `null` = front strictement plan. */
  wavefrontRadiusM?: number | null;
  /** `(Hz, dB)` au bord du secteur, relatifs à l'axe. */
  edgeLevelDb: [number, number][];
}

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
  /** Secteur encore rayonné par un guide **isophase**, degrés. 0 = front
   * parfaitement plan. C'est lui qui décale l'angle de raccord vers une caisse
   * à guide courbé, par tangence des deux fronts. */
  wgIsophaseSectorDeg?: number;
  /** Relevé d'identification du guide, quand il existe. */
  guideMeasurement?: GuideMeasurement | null;
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
  /** Direction du pull-back automatique (degrés, convention §2 : **180° =
   * vers le haut**), si le bumper et sa barre ne suffisent plus. Doit rester
   * dans 180° ± `Settings.pullBackToleranceDeg`. `null` tant que
   * l'utilisateur n'a pas choisi : le solveur suggère alors la verticale. */
  pullBackAngle?: number | null;
  /** Pull-back activé à la main, pour répartir la charge quand les points
   * d'accroche sont faibles. Demande une assiette imposée. Sans effet quand le
   * pull-back est de toute façon obligatoire, et en stack. */
  pullBackEnabled?: boolean;
  /** Pull-back manuel : tension voulue, N (même base que les efforts
   * affichés, poids × k_dyn). Avec la direction et l'assiette imposée, elle
   * fixe l'accroche. Ramenée dans `BumperView.pullBackTensionRangeN`. `null` :
   * la moitié de la charge au pull-back. */
  manualPullBackTensionN?: number | null;
  /** Vol : famille d'accroche et nombre de points. Le trou est toujours
   * choisi par le solveur. Absent : Auto, 1 point. */
  rigging?: RiggingRequest;
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

/** Perçage des deux pions du bumper, coté depuis ses bords comme sur le plan.
 * C'est du perçage déclaré : ni déduit de la quincaillerie de l'enceinte, ni
 * reconstruit à l'écran. */
export interface BumperPins {
  /** Recul du pion avant depuis la face avant, mm. */
  frontFromFrontMm: number;
  /** Avancée du pion arrière depuis la face arrière, mm. */
  rearFromRearMm: number;
  /** Hauteur des deux pions au-dessus du dessous du bumper, mm. Cote de
   * dessin et de stack : en vol, c'est la bielle qui place le bumper. */
  heightFromBottomMm: number;
}

/** Barre arrière du bumper : elle descend de son trou haut et se boulonne au
 * caisson de référence par sa paire ancrage/verrou, comme la barre arrière
 * d'une jonction. C'est son bras qui fixe ce que la paire encaisse.
 *
 * Elle existe en plusieurs longueurs, une par inclinaison — c'est ainsi qu'on
 * penche la première tête en stack. Le vol monte toujours celle à 0°. */
export interface BumperRearBar {
  /** Inclinaison de la barre par rapport au bumper, degrés. 0° en vol. */
  tiltDeg: number;
  /** Abscisse du trou haut depuis l'ancrage, le long de l'axe de la paire, mm. */
  topHoleAlongMm: number;
  /** Déport du trou haut en travers de cet axe, mm, positif vers l'avant. */
  topHoleLateralMm: number;
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
  pins: BumperPins;
  /** Longueur utile de la bielle de pivot avant, mm : entraxe entre la
   * charnière haute du caisson et la goupille haute, dans le bumper. Avec le
   * trou haut de la barre, c'est elle qui place le bumper en vol — le perçage
   * coté depuis les bords ne sert plus qu'au dessin et au stack. */
  pivotBarUsableLengthMm: number;
  /** Barres arrière disponibles, une par inclinaison. */
  rearBars: BumperRearBar[];
  /** Trous de manille et trous de liaison de la barre, repère bumper
   * (origine au centre du dessous, x vers l'arrière, y vers le haut). */
  rigging?: BumperRigging | null;
  compatibleSpeakers: BumperCompatibility[];
}

export interface BumperRigging {
  shackleHoles: [number, number][];
  barLinkHoles: [number, number][];
  wllKg: number;
}

export interface BumperBarGeometry {
  /** Repère barre : origine au centre, x le long de la barre, sens normal. */
  pickupHoles: [number, number][];
  linkPins: [[number, number], [number, number]];
  wllKg: number;
}

export type RiggingSupport = "auto" | "bumper" | "bar";

export interface RiggingRequest {
  support: RiggingSupport;
  /** 1 ou 2 moteurs. */
  points: number;
  /** Montage imposé, indice dans `RiggingView.barMounts`. */
  barMountIndex: number | null;
}

export interface BumperBarCompatibility {
  bumperModelId: string;
}

export interface BumperBarModel {
  id: string;
  name: string;
  schemaVersion: number;
  /** Portée max de déport depuis le centre du bumper, mm : au-delà, un pull-back prend le relais. */
  maxDeportMm: number;
  geometry?: BumperBarGeometry | null;
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
  /** Tolérance du pull-back autour de la verticale (degrés) : la direction
   * doit rester dans 180° ± cette valeur. 10° par défaut (Meyer Sound). */
  pullBackToleranceDeg: number;
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
  pullBackTensionN: number;
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
  /** Point d'accroche du pull-back sur l'enceinte du bas, repère global. */
  pullBackPointGlobal: Vec2 | null;
  /** Direction de traction du pull-back, repère global. */
  pullBackDirectionGlobal: Vec2 | null;
  /** Même direction, en degrés (convention §2 : 0° bas, 90° avant, 180° haut,
   * 270° arrière) : angle réel auquel ancrer le pull-back, dans 180° ±
   * tolérance (choisi par
   * l'utilisateur, ou suggéré par le solveur tant qu'il n'a pas encore
   * choisi — voir `BumperView.pullBackAngleRangeDeg`). */
  pullBackDirectionAngleDeg: number | null;
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
  /** Au-delà de la portée de la barre (`BumperBarModel.maxDeportMm`), elle ne suffit plus : un pull-back est
   * automatiquement mise en place (voir `pullBackTensionN`/`pullBackPointGlobal` sur
   * ClusterResult). */
  bumperBarExceeded: boolean;
  /** Plage de directions de traction physiquement valables (degrés,
   * convention §2), présente seulement si `bumperBarExceeded`. Le point d'ancrage
   * réel dépend du terrain : à choisir dedans, ce n'est pas à l'algorithme
   * de décider seul. */
  pullBackAngleRangeDeg: [number, number] | null;
  /** Pull-back manuel seulement : tensions saisissables, N. Elles
   * correspondent aux accroches atteignables sur le bumper et sa barre. */
  pullBackTensionRangeN: [number, number] | null;
  /** Part du poids dynamisé reprise par le pull-back, 0 sans pull-back. */
  pullBackLoadShare: number;
  /** Efforts transmis par le bumper à l'enceinte de référence, repère de
   * cette enceinte, par flanc. Vol uniquement. */
  orientationForceN: number | null;
  orientationAngleDeg: number | null;
  pivotForceN: number | null;
  pivotAngleDeg: number | null;
  /** Les deux mêmes en repère global, avec leur point d'application sur
   * l'enceinte de référence : le viewer les dessine tels quels. */
  /** Les deux pions à leur perçage déclaré (`BumperModel.pins`). Point
   * d'application des deux efforts ci-dessus : la statique se résout là où la
   * barre est boulonnée, et ces positions ne sortent que de là. */
  orientationPointGlobal: Vec2 | null;
  pivotPointGlobal: Vec2 | null;
  /** La paire ancrage/verrou par laquelle la barre du bumper est boulonnée sur
   * l'enceinte de référence. Vol uniquement : en stack le bumper est goupillé
   * sans barre, il n'y a pas de paire à encastrer. */
  pairAnchorHoleGlobal: Vec2 | null;
  pairLatchHoleGlobal: Vec2 | null;
  fPairAnchorGlobal: Vec2 | null;
  fPairLatchGlobal: Vec2 | null;
  fPairAnchorN: number | null;
  fPairAnchorAngleDeg: number | null;
  fPairLatchN: number | null;
  fPairLatchAngleDeg: number | null;
  /** Moment de la barre du bumper au barycentre de la paire, N·m par flanc. */
  pairMomentNm: number | null;
  orientationForceGlobal: Vec2 | null;
  pivotForceGlobal: Vec2 | null;
  /** Charge que le bumper reprend en entier : la manille en vol, la réaction du
   * sol en stack. **Pas** par flanc — une manille n'est pas doublée. */
  supportForceN: number;
  supportForceGlobal: Vec2;
  supportPointGlobal: Vec2;
  supportAngleDeg: number;
  /** Moment que la structure du bumper transfère entre ses deux pions, réduit à
   * leur milieu (N·m, par flanc). L'équivalent, pour le bumper, du moment de
   * barre d'une jonction. */
  pinPairMomentNm: number;
  /** Entraxe des deux pions, le bras de ce couple. */
  pinSpanMm: number;
  /** Vol, bumper aux trous déclarés : trous retenus et charge de chacun. */
  rigging: RiggingView | null;
}

export interface RiggingPointView {
  label: string;
  bumperXMm: number;
  pointGlobal: Vec2;
  tensionN: number;
  loadKg: number;
  wllKg: number;
  overloaded: boolean;
}

export interface BarMountView {
  label: string;
  flipped: boolean;
  centerXMm: number;
}

export interface RiggingView {
  /** Famille retenue, jamais "auto". */
  support: RiggingSupport;
  barMounts: BarMountView[];
  barMountIndex: number | null;
  points: RiggingPointView[];
  targetTiltDeg: number | null;
  achievedTiltDeg: number;
  tiltErrorDeg: number | null;
  bumperHolesGlobal: Vec2[];
  /** Les 4 trous de liaison de la barre sur le bumper, montés ou non. */
  bumperLinkHolesGlobal: Vec2[];
  barHolesGlobal: Vec2[];
  barPinsGlobal: Vec2[];
  /** Silhouette schématique de la barre montée, polygone fermé. */
  barOutlineGlobal: Vec2[];
  /** Effort de chaque patte sur son trou de liaison du bumper, même ordre
   * que `barPinsGlobal`. Charge entière, pas par flanc. */
  barLinkForces: LinkForceView[];
}

export interface LinkForceView {
  pointGlobal: Vec2;
  forceGlobal: Vec2;
  forceN: number;
  /** Repère de l'enceinte du haut (convention §2). */
  angleDeg: number;
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
  /** Hauteur de bouche **physique** D, mm : l'ARF du verdict, le profil de
   * retard, la géométrie. */
  radiatingHeightMm: number;
  /** Hauteur de bouche **acoustique** équivalente, mm — celle que voit le
   * rayonnement. Sert au critère 5 et à lui seul. `null` : on retombe sur la
   * bouche physique. Sans objet pour un guide à front courbé. */
  acousticMouthHeightMm?: number | null;
  /** Secteur encore rayonné par un guide isophase, degrés. 0 = front plan. */
  isophaseSectorDeg?: number;
  speakerCount: number;
  splaysDeg: number[];
  distancesM: number[];
  fMaxHz: number;
  guide: WstGuideKind;
}

export interface WstDerived {
  stepMm: number;
  gapMm: number;
  /** ARF du verdict : celui de la bouche physique. */
  arf: number;
  lineHeightM: number;
  /** Bouche réellement utilisée : celle de l'enceinte si elle en déclare une. */
  radiatingHeightMm: number;
  /** Bouche acoustique retenue, mm. Égale à la physique quand rien ne la
   * renseigne, ou pour un guide à front courbé où elle n'a pas de sens. */
  acousticMouthHeightMm: number;
  /** La bouche acoustique vient-elle d'un relevé, ou d'un repli sur la
   * physique ? L'écran doit pouvoir dire d'où sort le chiffre. */
  acousticMouthIsMeasured: boolean;
  /** Secteur du guide isophase en jeu, degrés. */
  isophaseSectorDeg: number;
  /** Guide réellement utilisé — l'enceinte l'emporte sur la saisie. */
  guide: WstGuideKind;
  /** Pas dérivé de la géométrie d'une enceinte (il varie alors avec l'angle). */
  stepFromSpeaker: boolean;
}

export interface WstCriterion1 {
  /** ARF de la bouche physique. **C'est lui qui fait le verdict** : le plus bas
   * des deux, donc le plus prudent. */
  arfGeometric: number;
  /** ARF de la bouche acoustique. Affiché à côté, jamais à la place. L'écart
   * entre les deux est la contribution de la diffraction de bride. */
  arfAcoustic: number;
  arfMin: number;
  /** Verdict, établi sur `arfGeometric` seul. */
  satisfied: boolean;
  /** `null` à ARF ≥ 1 : ligne continue, pas de lobe de réseau. */
  sideLobeAttenuationDb: number | null;
  /** Le même lobe lu sur l'ARF acoustique — la borne optimiste. */
  sideLobeAttenuationAcousticDb: number | null;
  axialLossDb: number;
  /** Les deux ARF diffèrent-ils ? Sinon la mention n'a pas d'objet. */
  arfsDiffer: boolean;
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
  /** Rayon du front relevé, m. `null` = front déclaré strictement plan. */
  wavefrontRadiusM: number | null;
  /** Écart au plan réel de ce front sur la bouche : `s = (D/2)² / (2R)`. */
  wavefrontDeviationMm: number | null;
  /** Fréquence jusqu'à laquelle cet écart tient dans λ/4. */
  isophaseFrequencyLimitHz: number | null;
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

/** Ce que le rayon réel du guide fait à un splay donné, face au rayon visé. */
export type WstRadiusVerdict = "tooCurved" | "tooFlat" | "matched";

export interface WstCurvedGuideRow {
  splayDeg: number;
  /** `θ_guide ≥ α` : les secteurs voisins se juxtaposent sans laisser de trou. */
  covered: boolean;
  /** `θ_guide − α`, **signé**. Positif = recouvrement, un réglage acceptable.
   * Négatif = trou angulaire, et ça c'est une faute. */
  overlapDeg: number;
  /** Premier creux dans la zone de recouvrement. `null` en trou angulaire :
   * sans zone commune, rien n'interfère. */
  overlapNotchHz: number | null;
  targetRadiusM: number | null;
  /** `R_réel − R_visé`, m. */
  radiusErrorM: number | null;
  radiusVerdict: WstRadiusVerdict | null;
  /** Fréquence au-dessus de laquelle la courbure doit être juste. Ce n'est PAS
   * une limite de fonctionnement : en dessous, des cordes plates approximent
   * l'arc à mieux que λ/4 et un guide isophase ferait pareil. */
  curvatureMattersAboveHz: number | null;
}

/** Le niveau au bord du secteur à une fréquence, et sa bosse au raccord. */
export interface WstEdgeLevelSample {
  frequencyHz: number;
  levelDb: number;
  /** Après médiane glissante : c'est lui qui entre dans le verdict. */
  smoothedLevelDb: number;
  spliceLevelDb: number;
  /** Accident étroit du guide : signalé, mais pas laissé juger la bande. */
  isNarrowArtifact: boolean;
}

export interface WstCurvedGuide {
  coverageDeg: number;
  maxSplayDeg: number;
  /** Plage recommandée : `[θ_guide − 3° ; θ_guide]`, jamais au-dessus. */
  recommendedSplayMinDeg: number;
  recommendedSplayMaxDeg: number;
  /** Médiane de bande du relevé quand il existe — pas son minimum. */
  levelAtHalfSplayDb: number;
  /** Bosse au raccord : deux secteurs voisins s'y somment en phase, soit
   * +6 dB. Cible 0 dB. */
  spliceLevelDb: number;
  edgeLevels: WstEdgeLevelSample[];
  hasNarrowArtifact: boolean;
  rows: WstCurvedGuideRow[];
  isophaseSectorDeg: number;
  /** Tangence des deux fronts : `(θ_iso + θ_courbe) / 2`. */
  transitionSplayDeg: number;
  /** Rayon réel du front, m, quand il a été identifié. */
  actualRadiusM: number | null;
  guideDelayProfile: WstGuideDelaySample[];
  actualDelayProfile: WstGuideDelaySample[];
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

/** Les chemins de charge de l'accrochage du bumper : le pendant de
 * `JointChecks` pour la liaison qui porte la grappe entière. */
export interface BumperChecks {
  utilizationFrontPin: number | null;
  utilizationRearPin: number | null;
  /** La paire où la barre du bumper est boulonnée. `null` en stack : le bumper
   * y est goupillé sans barre. */
  utilizationBarAnchor: number | null;
  utilizationBarLatch: number | null;
  utilizationWorst: number;
  governingPath: string | null;
}

export interface ClusterExport {
  definition: Cluster;
  result: ClusterResult;
  /** Un par jonction, même ordre que `result.joints`. */
  jointChecks: JointChecks[];
  /** L'accrochage du bumper : pions, et paire de sa barre en vol. */
  bumperChecks: BumperChecks;
  /** Le pire taux, jonctions **et** accrochage du bumper confondus. */
  utilizationWorst: number;
  /** Coefficient de sécurité réel de la grappe : `sf / pire taux`. */
  safetyFactor: number;
  /** Le même à k_dyn = 1,1, pour comparer à Soundvision qui ne dynamise pas de
   * la même façon. Les réglages ne sont pas modifiés pour autant. */
  safetyFactorStatic: number;
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

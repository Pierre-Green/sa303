//! Persistance JSON versionnée (brief §8) : un fichier par entité, plus un index.
//! Migration à la lecture si `schemaVersion` est inférieur au courant ; on ne
//! casse jamais silencieusement un fichier existant.

use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::settings::Settings;
use sa303_core::speaker::SpeakerModel;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub const SPEAKER_SCHEMA_VERSION: u32 = 3;
pub const CLUSTER_SCHEMA_VERSION: u32 = 2;
pub const BUMPER_SCHEMA_VERSION: u32 = 1;
pub const BUMPER_BAR_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EntityKind {
    /// Alias : anciennement `box`, avant de renommer "caisson" en "enceinte"
    /// ("speaker") dans tout le code — un index.json déjà sur disque avec
    /// l'ancienne valeur ne doit jamais faire planter la lecture (brief §8).
    #[serde(alias = "box")]
    Speaker,
    Cluster,
    Bumper,
    /// Alias : anciennement `bar`, avant de rattacher la barre de déport au
    /// domaine bumper ("bumper_bar", brief §8) — un index.json déjà sur
    /// disque avec l'ancienne valeur ne doit jamais faire planter la lecture.
    #[serde(rename = "bumper_bar", alias = "bar")]
    BumperBar,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct IndexEntry {
    id: String,
    name: String,
    kind: EntityKind,
    schema_version: u32,
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_data_dir().map_err(|e| e.to_string())
}

fn ensure_dir(dir: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("création de {}: {e}", dir.display()))
}

/// Le répertoire s'appelait `boxes` avant de renommer "caisson" en "enceinte"
/// ("speaker") dans tout le code : si un répertoire `speakers` n'existe pas
/// encore mais qu'un `boxes` si, on le renomme une bonne fois plutôt que de
/// perdre silencieusement l'accès aux enceintes déjà enregistrées (brief §8).
fn migrate_legacy_speakers_directory(app: &AppHandle) -> Result<(), String> {
    let base = data_dir(app)?;
    let legacy = base.join("boxes");
    let current = base.join("speakers");
    if legacy.exists() && !current.exists() {
        fs::rename(&legacy, &current).map_err(|e| {
            format!(
                "migration de {} vers {}: {e}",
                legacy.display(),
                current.display()
            )
        })?;
    }
    Ok(())
}

fn speakers_dir(app: &AppHandle) -> Result<PathBuf, String> {
    migrate_legacy_speakers_directory(app)?;
    let dir = data_dir(app)?.join("speakers");
    ensure_dir(&dir)?;
    Ok(dir)
}

fn clusters_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = data_dir(app)?.join("clusters");
    ensure_dir(&dir)?;
    Ok(dir)
}

fn bumpers_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = data_dir(app)?.join("bumpers");
    ensure_dir(&dir)?;
    Ok(dir)
}

/// Le répertoire s'appelait `bars` avant de rattacher la barre de déport au
/// domaine bumper ("bumper_bar", brief §8) : si un répertoire `bumper-bars`
/// n'existe pas encore mais qu'un `bars` si, on le renomme une bonne fois
/// plutôt que de perdre silencieusement l'accès aux barres déjà enregistrées.
fn migrate_legacy_bumper_bars_directory(app: &AppHandle) -> Result<(), String> {
    let base = data_dir(app)?;
    let legacy = base.join("bars");
    let current = base.join("bumper-bars");
    if legacy.exists() && !current.exists() {
        fs::rename(&legacy, &current).map_err(|e| {
            format!(
                "migration de {} vers {}: {e}",
                legacy.display(),
                current.display()
            )
        })?;
    }
    Ok(())
}

fn bumper_bars_dir(app: &AppHandle) -> Result<PathBuf, String> {
    migrate_legacy_bumper_bars_directory(app)?;
    let dir = data_dir(app)?.join("bumper-bars");
    ensure_dir(&dir)?;
    Ok(dir)
}

fn index_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = data_dir(app)?;
    ensure_dir(&dir)?;
    Ok(dir.join("index.json"))
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = data_dir(app)?;
    ensure_dir(&dir)?;
    Ok(dir.join("settings.json"))
}

fn read_index(app: &AppHandle) -> Result<Vec<IndexEntry>, String> {
    let path = index_path(app)?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("index corrompu: {e}"))
}

fn write_index(app: &AppHandle, entries: &[IndexEntry]) -> Result<(), String> {
    let path = index_path(app)?;
    let raw = serde_json::to_string_pretty(entries).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

/// Retire un élément du disque et de l'index, sans se demander à qui il
/// appartient : les appelants publics ont déjà tranché.
fn remove_entity(app: &AppHandle, kind: EntityKind, id: &str) -> Result<(), String> {
    let dir = match kind {
        EntityKind::Speaker => speakers_dir(app)?,
        EntityKind::Bumper => bumpers_dir(app)?,
        EntityKind::BumperBar => bumper_bars_dir(app)?,
        EntityKind::Cluster => clusters_dir(app)?,
    };
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    remove_from_index(app, id, kind)
}

fn upsert_index(app: &AppHandle, entry: IndexEntry) -> Result<(), String> {
    let mut entries = read_index(app)?;
    match entries
        .iter_mut()
        .find(|e| e.id == entry.id && e.kind == entry.kind)
    {
        Some(existing) => *existing = entry,
        None => entries.push(entry),
    }
    write_index(app, &entries)
}

fn remove_from_index(app: &AppHandle, id: &str, kind: EntityKind) -> Result<(), String> {
    let mut entries = read_index(app)?;
    entries.retain(|e| !(e.id == id && e.kind == kind));
    write_index(app, &entries)
}

/// Grille de trous des SA303 **avant** la modification d'accastillage. Figée :
/// une migration doit rendre le même résultat pour toujours, elle ne peut donc
/// pas suivre une constante qui bouge.
const LEGACY_SA303_SPLAY_GRID: [f64; 14] = [
    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 12.0, 15.0, 20.0,
];

/// Grille de trous après la modification : arc court (couronne intérieure,
/// splays impairs) et arc long (extérieure, pairs) réunis. Figée elle aussi.
const SA303_SPLAY_GRID_V3: [f64; 17] = [
    0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 8.0, 9.0, 10.0, 11.0, 12.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0,
];

fn same_grid(grid: &[f64], reference: &[f64]) -> bool {
    grid.len() == reference.len()
        && grid
            .iter()
            .zip(reference)
            .all(|(a, b)| (a - b).abs() < 1e-9)
}

fn migrate_speaker_model(mut s: SpeakerModel) -> SpeakerModel {
    // v1 → v2 : le front rayonné et le renommage de la bouche du guide sont
    // traités en amont, sur le JSON brut (`migrate_legacy_speaker_json`), parce
    // qu'ils ne s'expriment pas sur une valeur déjà désérialisée.
    //
    // v2 → v3 : l'accastillage a changé, les trous de réglage d'orientation ne
    // sont plus les mêmes. Sans cette reprise, une enceinte déjà sur disque
    // continuerait d'offrir des angles qui n'existent plus (6° et 7°) et
    // d'ignorer ceux qui viennent d'apparaître. La condition d'égalité est
    // volontairement stricte : une grille personnalisée n'est jamais écrasée,
    // seules les fiches encore conformes à l'ancien SA303 sont reprises.
    if same_grid(&s.mechanical.splay_grid, &LEGACY_SA303_SPLAY_GRID) {
        s.mechanical.splay_grid = SA303_SPLAY_GRID_V3.to_vec();
    }
    s.schema_version = SPEAKER_SCHEMA_VERSION;
    s
}

/// v1 → v2 : le front rayonné (`wgFront`) n'existait pas. Le déduire de
/// l'ouverture verticale est fidèle à ce que décrivaient les fiches : une
/// caisse à secteur propre est à courbure constante, une caisse sans secteur
/// est isophase et n'ouvre que par le splay mécanique. Sans cette déduction,
/// une CCA enregistrée passerait silencieusement pour un front plan et se
/// verrait appliquer un critère qui ne la concerne pas.
///
/// Le renommage `radiatingHeight` → `wgOutputHeight` est couvert par un
/// `#[serde(alias)]` et n'a donc rien à faire ici.
fn migrate_legacy_speaker_json(mut value: serde_json::Value) -> serde_json::Value {
    let Some(acoustics) = value.get_mut("acoustics").and_then(|a| a.as_object_mut()) else {
        return value;
    };
    if acoustics.contains_key("wgFront") {
        return value;
    }
    let has_own_sector = acoustics
        .get("directivityVertical")
        .and_then(|d| d.as_f64())
        .is_some_and(|d| d > 0.0);
    let front = if has_own_sector {
        "constantCurvature"
    } else {
        "isophase"
    };
    acoustics.insert("wgFront".into(), serde_json::Value::String(front.into()));
    value
}

fn migrate_cluster(mut c: Cluster) -> Cluster {
    c.schema_version = CLUSTER_SCHEMA_VERSION;
    c
}

fn migrate_bumper_model(mut b: BumperModel) -> BumperModel {
    b.schema_version = BUMPER_SCHEMA_VERSION;
    b
}

fn migrate_bumper_bar_model(mut b: BumperBarModel) -> BumperBarModel {
    b.schema_version = BUMPER_BAR_SCHEMA_VERSION;
    b
}

pub fn load_speaker_model(app: &AppHandle, id: &str) -> Result<SpeakerModel, String> {
    let path = speakers_dir(app)?.join(format!("{id}.json"));
    let raw = fs::read_to_string(&path).map_err(|e| format!("enceinte {id} introuvable: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let parsed: SpeakerModel =
        serde_json::from_value(migrate_legacy_speaker_json(value)).map_err(|e| e.to_string())?;
    if parsed.schema_version < SPEAKER_SCHEMA_VERSION {
        let migrated = migrate_speaker_model(parsed);
        save_speaker_model(app, &migrated)?;
        Ok(migrated)
    } else {
        Ok(parsed)
    }
}

/// Écrit sans poser de question : sert aussi bien à l'enregistrement demandé
/// par l'utilisateur qu'à l'installation du catalogue de base, qui doit
/// pouvoir écraser un fichier existant.
fn write_speaker_model(app: &AppHandle, id: &str, s: &SpeakerModel) -> Result<(), String> {
    let path = speakers_dir(app)?.join(format!("{id}.json"));
    let raw = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())?;
    upsert_index(
        app,
        IndexEntry {
            id: s.id.clone(),
            name: s.name.clone(),
            kind: EntityKind::Speaker,
            schema_version: s.schema_version,
        },
    )
}

pub fn save_speaker_model(app: &AppHandle, s: &SpeakerModel) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Speaker, &s.id)?;
    write_speaker_model(app, &s.id.clone(), s)
}

pub fn delete_speaker_model(app: &AppHandle, id: &str) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Speaker, id)?;
    remove_entity(app, EntityKind::Speaker, id)
}

pub fn list_speaker_models(app: &AppHandle) -> Result<Vec<SpeakerModel>, String> {
    let dir = speakers_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            let id = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push(load_speaker_model(app, &id)?);
        }
    }
    Ok(out)
}

pub fn load_bumper_model(app: &AppHandle, id: &str) -> Result<BumperModel, String> {
    let path = bumpers_dir(app)?.join(format!("{id}.json"));
    let raw = fs::read_to_string(&path).map_err(|e| format!("bumper {id} introuvable: {e}"))?;
    let parsed: BumperModel = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if parsed.schema_version < BUMPER_SCHEMA_VERSION {
        let migrated = migrate_bumper_model(parsed);
        save_bumper_model(app, &migrated)?;
        Ok(migrated)
    } else {
        Ok(parsed)
    }
}

/// Écrit sans poser de question : sert aussi bien à l'enregistrement demandé
/// par l'utilisateur qu'à l'installation du catalogue de base, qui doit
/// pouvoir écraser un fichier existant.
fn write_bumper_model(app: &AppHandle, id: &str, b: &BumperModel) -> Result<(), String> {
    let path = bumpers_dir(app)?.join(format!("{id}.json"));
    let raw = serde_json::to_string_pretty(b).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())?;
    upsert_index(
        app,
        IndexEntry {
            id: b.id.clone(),
            name: b.name.clone(),
            kind: EntityKind::Bumper,
            schema_version: b.schema_version,
        },
    )
}

pub fn save_bumper_model(app: &AppHandle, b: &BumperModel) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Bumper, &b.id)?;
    write_bumper_model(app, &b.id.clone(), b)
}

pub fn delete_bumper_model(app: &AppHandle, id: &str) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Bumper, id)?;
    remove_entity(app, EntityKind::Bumper, id)
}

pub fn list_bumper_models(app: &AppHandle) -> Result<Vec<BumperModel>, String> {
    let dir = bumpers_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            let id = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push(load_bumper_model(app, &id)?);
        }
    }
    Ok(out)
}

pub fn load_bumper_bar_model(app: &AppHandle, id: &str) -> Result<BumperBarModel, String> {
    let path = bumper_bars_dir(app)?.join(format!("{id}.json"));
    let raw = fs::read_to_string(&path).map_err(|e| format!("barre {id} introuvable: {e}"))?;
    let parsed: BumperBarModel = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    if parsed.schema_version < BUMPER_BAR_SCHEMA_VERSION {
        let migrated = migrate_bumper_bar_model(parsed);
        save_bumper_bar_model(app, &migrated)?;
        Ok(migrated)
    } else {
        Ok(parsed)
    }
}

/// Écrit sans poser de question : sert aussi bien à l'enregistrement demandé
/// par l'utilisateur qu'à l'installation du catalogue de base, qui doit
/// pouvoir écraser un fichier existant.
fn write_bumper_bar_model(app: &AppHandle, id: &str, b: &BumperBarModel) -> Result<(), String> {
    let path = bumper_bars_dir(app)?.join(format!("{id}.json"));
    let raw = serde_json::to_string_pretty(b).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())?;
    upsert_index(
        app,
        IndexEntry {
            id: b.id.clone(),
            name: b.name.clone(),
            kind: EntityKind::BumperBar,
            schema_version: b.schema_version,
        },
    )
}

pub fn save_bumper_bar_model(app: &AppHandle, b: &BumperBarModel) -> Result<(), String> {
    refuse_if_builtin(EntityKind::BumperBar, &b.id)?;
    write_bumper_bar_model(app, &b.id.clone(), b)
}

pub fn delete_bumper_bar_model(app: &AppHandle, id: &str) -> Result<(), String> {
    refuse_if_builtin(EntityKind::BumperBar, id)?;
    remove_entity(app, EntityKind::BumperBar, id)
}

pub fn list_bumper_bar_models(app: &AppHandle) -> Result<Vec<BumperBarModel>, String> {
    let dir = bumper_bars_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            let id = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push(load_bumper_bar_model(app, &id)?);
        }
    }
    Ok(out)
}

/// Une grappe était homogène : un seul `speakerModelId` pour toute la chaîne.
/// Elle est désormais hétérogène (`speakerModelIds`, une entrée par enceinte).
/// Le changement est structurel — un simple `serde(alias)` ne peut pas
/// transformer une chaîne en tableau — donc on remodèle le JSON brut avant de
/// le désérialiser : l'ancien id est répété autant de fois qu'il y a
/// d'enceintes. Un fichier déjà sur disque ne doit jamais devenir illisible
/// (brief §8).
pub(crate) fn migrate_legacy_cluster_json(mut value: serde_json::Value) -> serde_json::Value {
    let Some(object) = value.as_object_mut() else {
        return value;
    };
    if object.contains_key("speakerModelIds") {
        return value;
    }
    let Some(legacy_id) = object
        .remove("speakerModelId")
        .or_else(|| object.remove("boxModelId"))
    else {
        return value;
    };
    let speaker_count = object
        .get("joints")
        .and_then(|j| j.as_array())
        .map_or(1, |joints| joints.len() + 1);
    let ids = vec![legacy_id; speaker_count];
    object.insert("speakerModelIds".into(), serde_json::Value::Array(ids));
    value
}

pub fn load_cluster(app: &AppHandle, id: &str) -> Result<Cluster, String> {
    let path = clusters_dir(app)?.join(format!("{id}.json"));
    let raw = fs::read_to_string(&path).map_err(|e| format!("grappe {id} introuvable: {e}"))?;
    let value: serde_json::Value = serde_json::from_str(&raw).map_err(|e| e.to_string())?;
    let parsed: Cluster =
        serde_json::from_value(migrate_legacy_cluster_json(value)).map_err(|e| e.to_string())?;
    if parsed.schema_version < CLUSTER_SCHEMA_VERSION {
        let migrated = migrate_cluster(parsed);
        save_cluster(app, &migrated)?;
        Ok(migrated)
    } else {
        Ok(parsed)
    }
}

/// Écrit sans poser de question : sert aussi bien à l'enregistrement demandé
/// par l'utilisateur qu'à l'installation du catalogue de base, qui doit
/// pouvoir écraser un fichier existant.
fn write_cluster(app: &AppHandle, id: &str, c: &Cluster) -> Result<(), String> {
    let path = clusters_dir(app)?.join(format!("{id}.json"));
    let raw = serde_json::to_string_pretty(c).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())?;
    upsert_index(
        app,
        IndexEntry {
            id: c.id.clone(),
            name: c.name.clone(),
            kind: EntityKind::Cluster,
            schema_version: c.schema_version,
        },
    )
}

pub fn save_cluster(app: &AppHandle, c: &Cluster) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Cluster, &c.id)?;
    write_cluster(app, &c.id.clone(), c)
}

pub fn delete_cluster(app: &AppHandle, id: &str) -> Result<(), String> {
    refuse_if_builtin(EntityKind::Cluster, id)?;
    remove_entity(app, EntityKind::Cluster, id)
}

pub fn list_clusters(app: &AppHandle) -> Result<Vec<Cluster>, String> {
    let dir = clusters_dir(app)?;
    let mut out = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.path().extension().and_then(|e| e.to_str()) == Some("json") {
            let id = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            out.push(load_cluster(app, &id)?);
        }
    }
    Ok(out)
}

pub fn load_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(crate::seed::default_settings());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let path = settings_path(app)?;
    let raw = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(path, raw).map_err(|e| e.to_string())
}

/// Ce qu'une version du logiciel a installé dans le dossier de données. Écrit
/// à côté des catalogues, relu au lancement suivant : c'est lui qui permet de
/// distinguer « composant de base retiré par une mise à jour » de « créé par
/// l'utilisateur », donc de nettoyer sans jamais toucher aux fichiers de
/// l'utilisateur.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct BuiltinIds {
    pub speakers: Vec<String>,
    pub bumpers: Vec<String>,
    pub bumper_bars: Vec<String>,
    pub clusters: Vec<String>,
}

impl BuiltinIds {
    fn contains(&self, kind: EntityKind, id: &str) -> bool {
        let ids = match kind {
            EntityKind::Speaker => &self.speakers,
            EntityKind::Bumper => &self.bumpers,
            EntityKind::BumperBar => &self.bumper_bars,
            EntityKind::Cluster => &self.clusters,
        };
        ids.iter().any(|known| known == id)
    }
}

fn builtins_manifest_path(app: &AppHandle) -> Result<PathBuf, String> {
    // Crée le dossier plutôt que de compter sur l'écriture d'un asset pour
    // l'avoir fait : sur une installation vierge, un catalogue vide suffirait
    // sinon à faire échouer l'écriture du manifeste.
    let dir = data_dir(app)?;
    ensure_dir(&dir)?;
    Ok(dir.join("builtins.json"))
}

/// Identifiants livrés par la version en cours d'exécution.
pub fn builtin_ids() -> BuiltinIds {
    BuiltinIds {
        speakers: crate::seed::builtin_speakers()
            .into_iter()
            .map(|b| b.id)
            .collect(),
        bumpers: crate::seed::builtin_bumpers()
            .into_iter()
            .map(|b| b.id)
            .collect(),
        bumper_bars: crate::seed::builtin_bumper_bars()
            .into_iter()
            .map(|b| b.id)
            .collect(),
        clusters: crate::seed::builtin_clusters()
            .into_iter()
            .map(|b| b.id)
            .collect(),
    }
}

/// Un composant de base ne s'enregistre ni ne se supprime depuis
/// l'application : il appartient au logiciel, seule une mise à jour le fait
/// bouger. Refuser explicitement plutôt que d'écrire un fichier qui serait
/// écrasé au prochain lancement — un enregistrement silencieusement annulé
/// serait pire que pas d'enregistrement du tout (brief §11.6).
fn refuse_if_builtin(kind: EntityKind, id: &str) -> Result<(), String> {
    if builtin_ids().contains(kind, id) {
        let what = match kind {
            EntityKind::Speaker => "Cette enceinte",
            EntityKind::Bumper => "Ce bumper",
            EntityKind::BumperBar => "Cette barre",
            EntityKind::Cluster => "Cette grappe",
        };
        return Err(format!(
            "{what} fait partie du catalogue livré avec le logiciel : elle ne peut être ni modifiée ni supprimée ici. Duplique-la pour partir de sa configuration."
        ));
    }
    Ok(())
}

/// Ce qu'une version précédente avait installé et que celle-ci ne livre plus.
///
/// C'est la seule liste que la synchronisation a le droit de supprimer. Elle se
/// calcule par différence contre le manifeste, jamais contre le contenu du
/// dossier : un élément absent du manifeste a été créé par l'utilisateur, et il
/// n'appartient pas au logiciel de l'effacer (brief §8).
fn stale_builtins(previous: &BuiltinIds, current: &BuiltinIds) -> Vec<(EntityKind, String)> {
    [
        (EntityKind::Speaker, &previous.speakers),
        (EntityKind::Bumper, &previous.bumpers),
        (EntityKind::BumperBar, &previous.bumper_bars),
        (EntityKind::Cluster, &previous.clusters),
    ]
    .into_iter()
    .flat_map(|(kind, ids)| {
        ids.iter()
            .filter(move |id| !current.contains(kind, id))
            .map(move |id| (kind, id.clone()))
    })
    .collect()
}

/// Installe le catalogue de base et le tient à jour.
///
/// Appelée à **chaque** lancement, pas seulement sur une installation vierge :
/// c'est ce qui fait qu'une mise à jour du logiciel propage ses corrections
/// dans un dossier de données existant, sans qu'il faille l'effacer. Les
/// éléments livrés sont réécrits tels quels ; ceux qu'une version précédente
/// avait installés et qui ne sont plus livrés sont retirés — et uniquement
/// ceux-là, d'où le manifeste. Tout ce que l'utilisateur a créé est laissé
/// intact.
pub fn sync_builtins(app: &AppHandle) -> Result<(), String> {
    let previous: BuiltinIds = match fs::read_to_string(builtins_manifest_path(app)?) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        // Première installation, ou manifeste illisible : on réinstalle tout et
        // on ne supprime rien. Ne jamais deviner une suppression.
        Err(_) => BuiltinIds::default(),
    };
    let current = builtin_ids();

    for builtin in crate::seed::builtin_speakers() {
        write_speaker_model(app, &builtin.id, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_bumpers() {
        write_bumper_model(app, &builtin.id, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_bumper_bars() {
        write_bumper_bar_model(app, &builtin.id, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_clusters() {
        write_cluster(app, &builtin.id, &builtin.model)?;
    }

    for (kind, id) in stale_builtins(&previous, &current) {
        remove_entity(app, kind, &id)?;
    }

    let raw = serde_json::to_string_pretty(&current).map_err(|e| e.to_string())?;
    fs::write(builtins_manifest_path(app)?, raw).map_err(|e| e.to_string())
}

/// Peuple le répertoire de données au premier lancement, puis installe et met à
/// jour le catalogue de base (brief §8).
pub fn ensure_seeded(app: &AppHandle) -> Result<(), String> {
    // Les réglages ne font pas partie du catalogue : l'utilisateur les ajuste,
    // donc on ne les écrase jamais — on les crée seulement s'ils manquent.
    if !settings_path(app)?.exists() {
        save_settings(app, &crate::seed::default_settings())?;
    }
    sync_builtins(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sa303_core::speaker::WaveguideFront;

    /// Le JSON d'une enceinte enregistrée avant la v2 : ni `wgFront`, et la
    /// bouche du guide encore sous son ancien nom.
    fn legacy_speaker_json(directivity_vertical: f64) -> serde_json::Value {
        serde_json::json!({
            "id": "sa303-cca",
            "name": "SA303-CCA",
            "schemaVersion": 1,
            "depth": 700.0,
            "height": 550.0,
            "totalVerticalAngle": 20.0,
            "massKg": 83.695,
            "cg": [10.84, 11.91],
            "hinge": { "x": -338.43, "y": 257.13, "jointSeparation": 552.384 },
            "crown": { "radius": 680.0, "delta": 20.0, "anchorAngle": 2.5, "splay0Angle": 5.0 },
            "splayGrid": [0.0, 5.0, 10.0],
            "frameHoleSplay": 0.0,
            "acoustics": {
                "fs": 60.0,
                "directivityHorizontal": 90.0,
                "directivityVertical": directivity_vertical,
                "radiatingHeight": 470.0
            }
        })
    }

    #[test]
    fn a_speaker_with_its_own_sector_migrates_to_a_constant_curvature_front() {
        // Sans cette déduction, une CCA enregistrée passerait pour un front plan
        // et se verrait appliquer un critère qui ne la concerne pas.
        let migrated = migrate_legacy_speaker_json(legacy_speaker_json(20.0));
        let speaker: SpeakerModel = serde_json::from_value(migrated).unwrap();
        assert_eq!(
            speaker.acoustics.wg_front,
            WaveguideFront::ConstantCurvature
        );
        assert_eq!(speaker.acoustics.wg_output_height, 470.0);
    }

    #[test]
    fn a_speaker_without_a_sector_migrates_to_a_flat_front() {
        let migrated = migrate_legacy_speaker_json(legacy_speaker_json(0.0));
        let speaker: SpeakerModel = serde_json::from_value(migrated).unwrap();
        assert_eq!(speaker.acoustics.wg_front, WaveguideFront::Isophase);
    }

    #[test]
    fn the_hardware_change_reaches_speakers_already_on_disk() {
        // Le seed ne s'exécute que sur une installation vierge : sans cette
        // reprise, une fiche déjà enregistrée continuerait d'offrir 6° et 7°,
        // qui ne sont plus percés.
        let mut speaker: SpeakerModel =
            serde_json::from_value(migrate_legacy_speaker_json(legacy_speaker_json(20.0))).unwrap();
        speaker.mechanical.splay_grid = LEGACY_SA303_SPLAY_GRID.to_vec();

        let migrated = migrate_speaker_model(speaker);
        assert_eq!(migrated.schema_version, SPEAKER_SCHEMA_VERSION);
        assert_eq!(migrated.mechanical.splay_grid, SA303_SPLAY_GRID_V3.to_vec());
        assert!(!migrated.mechanical.splay_grid.contains(&6.0));
        assert!(migrated.mechanical.splay_grid.contains(&17.0));
    }

    #[test]
    fn a_custom_splay_grid_is_never_overwritten() {
        // Une enceinte percée autrement n'a rien à voir avec le SA303 : la
        // migration doit la laisser telle quelle.
        let custom = vec![0.0, 5.0, 10.0];
        let mut speaker: SpeakerModel =
            serde_json::from_value(migrate_legacy_speaker_json(legacy_speaker_json(20.0))).unwrap();
        speaker.mechanical.splay_grid = custom.clone();

        assert_eq!(migrate_speaker_model(speaker).mechanical.splay_grid, custom);
    }

    #[test]
    fn the_seeded_grid_is_exactly_the_two_arcs() {
        // Le seed et la migration décrivent le même accastillage : s'ils
        // divergeaient, deux installations donneraient deux enceintes.
        assert_eq!(
            crate::seed::sa303_splay_grid(),
            SA303_SPLAY_GRID_V3.to_vec()
        );
    }

    #[test]
    fn every_hole_sits_on_the_arc_its_parity_implies() {
        // La géométrie choisit le rayon de couronne sur la parité du splay. Si
        // un trou de l'arc court devenait pair, elle irait chercher le mauvais
        // rayon sans rien signaler.
        use sa303_core::speaker::is_odd_splay;
        assert!(crate::seed::SA303_SHORT_ARC_SPLAYS
            .iter()
            .all(|&s| is_odd_splay(s)));
        assert!(crate::seed::SA303_LONG_ARC_SPLAYS
            .iter()
            .all(|&s| !is_odd_splay(s)));
    }

    /// Les grappes d'exemple sont des fichiers déposés à la main : rien ne les
    /// relit avant le premier lancement chez un utilisateur. Ce test est ce
    /// relecteur — il échoue en CI plutôt que de livrer un asset cassé.
    #[test]
    fn every_seeded_cluster_asset_describes_a_buildable_cluster() {
        use sa303_core::compute_cluster;

        let clusters: Vec<_> = crate::seed::builtin_clusters()
            .into_iter()
            .map(|b| b.model)
            .collect();
        assert!(!clusters.is_empty(), "aucune grappe d'exemple embarquée");

        let speakers: Vec<_> = crate::seed::builtin_speakers()
            .into_iter()
            .map(|b| b.model)
            .collect();
        let bumper = crate::seed::builtin_bumpers()
            .into_iter()
            .map(|b| b.model)
            .next()
            .expect("un bumper livré");
        let bumper_bars: Vec<_> = crate::seed::builtin_bumper_bars()
            .into_iter()
            .map(|b| b.model)
            .collect();
        let settings = crate::seed::default_settings();

        let mut ids = std::collections::HashSet::new();
        for cluster in &clusters {
            assert!(
                ids.insert(cluster.id.clone()),
                "deux exemples partagent l'id \"{}\" : le second écraserait le premier au seed",
                cluster.id
            );
            assert_eq!(
                cluster.speaker_model_ids.len(),
                cluster.joints.len() + 1,
                "\"{}\" : autant d'enceintes que de jonctions + 1",
                cluster.name
            );
            // Le vrai contrôle : chaque exemple doit passer le solveur. Un angle
            // non percé ou une jonction non déclarée s'y verrait tout de suite.
            compute_cluster(&speakers, cluster, &settings, &bumper, &bumper_bars).unwrap_or_else(
                |e| {
                    panic!(
                        "grappe d'exemple \"{}\" non calculable : {}",
                        cluster.name, e.reason
                    )
                },
            );
        }
    }

    fn ids(speakers: &[&str], clusters: &[&str]) -> BuiltinIds {
        BuiltinIds {
            speakers: speakers.iter().map(|s| s.to_string()).collect(),
            clusters: clusters.iter().map(|s| s.to_string()).collect(),
            ..BuiltinIds::default()
        }
    }

    #[test]
    fn an_update_that_drops_a_component_removes_it_and_nothing_else() {
        // Version précédente : deux enceintes et deux grappes livrées.
        let previous = ids(&["sa303-isophase", "sa303-retire"], &["demo-a", "demo-b"]);
        // Cette version ne livre plus « sa303-retire » ni « demo-b ».
        let current = ids(&["sa303-isophase"], &["demo-a"]);

        let stale = stale_builtins(&previous, &current);
        assert_eq!(stale.len(), 2, "obtenu {stale:?}");
        assert!(stale.contains(&(EntityKind::Speaker, "sa303-retire".into())));
        assert!(stale.contains(&(EntityKind::Cluster, "demo-b".into())));
    }

    #[test]
    fn what_the_user_created_is_never_a_candidate_for_removal() {
        // Le manifeste ne connaît que ce que le logiciel a installé. Une grappe
        // créée par l'utilisateur n'y figure pas, donc elle ne peut pas
        // apparaître dans la liste des retraits, même si le logiciel n'en livre
        // aucune : c'est ce qui garantit qu'une mise à jour ne touche pas au
        // travail de l'utilisateur (brief §8).
        let previous = ids(&[], &["demo-a"]);
        let current = BuiltinIds::default();
        let stale = stale_builtins(&previous, &current);
        assert_eq!(stale, vec![(EntityKind::Cluster, "demo-a".into())]);
    }

    #[test]
    fn a_first_install_removes_nothing() {
        // Pas de manifeste : on installe et on ne devine aucune suppression.
        let stale = stale_builtins(&BuiltinIds::default(), &builtin_ids());
        assert!(stale.is_empty(), "obtenu {stale:?}");
    }

    #[test]
    fn relaunching_the_same_version_removes_nothing() {
        let current = builtin_ids();
        assert!(stale_builtins(&current, &current).is_empty());
    }

    #[test]
    fn a_shipped_component_cannot_be_saved_or_deleted_from_the_app() {
        // Sans ce refus, l'enregistrement réussirait puis serait écrasé au
        // lancement suivant : une modification perdue en silence, exactement ce
        // que le brief interdit (§11.6).
        let shipped = builtin_ids();
        let speaker = shipped.speakers.first().expect("une enceinte livrée");
        let err = refuse_if_builtin(EntityKind::Speaker, speaker)
            .expect_err("un composant livré doit être refusé");
        assert!(err.contains("catalogue"), "message obtenu : {err}");

        // Un identifiant inconnu du catalogue reste libre.
        refuse_if_builtin(EntityKind::Speaker, "une-enceinte-a-moi").expect("id utilisateur");
    }

    #[test]
    fn every_asset_is_named_after_the_id_it_declares() {
        // Le fichier écrit dans le dossier de données est nommé d'après le nom
        // de l'asset, alors que l'index et les références de grappe utilisent le
        // champ `id` du JSON. S'ils divergeaient, l'élément serait introuvable
        // par son propre identifiant.
        for b in crate::seed::builtin_speakers() {
            assert_eq!(b.id, b.model.id, "speakers/{}.json", b.id);
        }
        for b in crate::seed::builtin_bumpers() {
            assert_eq!(b.id, b.model.id, "bumpers/{}.json", b.id);
        }
        for b in crate::seed::builtin_bumper_bars() {
            assert_eq!(b.id, b.model.id, "bumper-bars/{}.json", b.id);
        }
        for b in crate::seed::builtin_clusters() {
            assert_eq!(b.id, b.model.id, "clusters/{}.json", b.id);
        }
    }

    #[test]
    fn a_front_already_declared_is_never_overwritten() {
        // Une enceinte à front plan peut très bien déclarer une ouverture
        // verticale : la déduction ne doit s'appliquer qu'en son absence.
        let mut value = legacy_speaker_json(20.0);
        value["acoustics"]["wgFront"] = serde_json::json!("isophase");
        let migrated = migrate_legacy_speaker_json(value);
        let speaker: SpeakerModel = serde_json::from_value(migrated).unwrap();
        assert_eq!(speaker.acoustics.wg_front, WaveguideFront::Isophase);
    }
}

//! Persistance JSON : un fichier par élément, rangé par famille dans le
//! dossier de données. Un fichier qui ne correspond pas au modèle courant est
//! une erreur de lecture, jamais une migration silencieuse.

use sa303_core::bumper::{BumperBarModel, BumperModel};
use sa303_core::cluster::Cluster;
use sa303_core::settings::Settings;
use sa303_core::speaker::SpeakerModel;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Speaker,
    Cluster,
    Bumper,
    BumperBar,
}

impl EntityKind {
    /// Dossier de la famille, sous le dossier de données — le même nom que
    /// dans `assets/`.
    fn dir_name(self) -> &'static str {
        match self {
            EntityKind::Speaker => "speakers",
            EntityKind::Cluster => "clusters",
            EntityKind::Bumper => "bumpers",
            EntityKind::BumperBar => "bumper-bars",
        }
    }

    /// « Cette enceinte », « Ce bumper »… pour les messages.
    fn this(self) -> &'static str {
        match self {
            EntityKind::Speaker => "Cette enceinte",
            EntityKind::Cluster => "Cette grappe",
            EntityKind::Bumper => "Ce bumper",
            EntityKind::BumperBar => "Cette barre",
        }
    }
}

/// Un élément persisté : un fichier `<id>.json` dans le dossier de sa famille.
pub trait Entity: Serialize + DeserializeOwned {
    const KIND: EntityKind;
    fn id(&self) -> &str;
}

impl Entity for SpeakerModel {
    const KIND: EntityKind = EntityKind::Speaker;
    fn id(&self) -> &str {
        &self.id
    }
}

impl Entity for Cluster {
    const KIND: EntityKind = EntityKind::Cluster;
    fn id(&self) -> &str {
        &self.id
    }
}

impl Entity for BumperModel {
    const KIND: EntityKind = EntityKind::Bumper;
    fn id(&self) -> &str {
        &self.id
    }
}

impl Entity for BumperBarModel {
    const KIND: EntityKind = EntityKind::BumperBar;
    fn id(&self) -> &str {
        &self.id
    }
}

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    ensure_dir(&dir)?;
    Ok(dir)
}

fn ensure_dir(dir: &PathBuf) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| format!("création de {}: {e}", dir.display()))
}

fn entity_dir(app: &AppHandle, kind: EntityKind) -> Result<PathBuf, String> {
    let dir = data_dir(app)?.join(kind.dir_name());
    ensure_dir(&dir)?;
    Ok(dir)
}

fn entity_path(app: &AppHandle, kind: EntityKind, id: &str) -> Result<PathBuf, String> {
    Ok(entity_dir(app, kind)?.join(format!("{id}.json")))
}

pub fn load<T: Entity>(app: &AppHandle, id: &str) -> Result<T, String> {
    let path = entity_path(app, T::KIND, id)?;
    let raw = fs::read_to_string(&path).map_err(|e| format!("{} introuvable : {e}", path.display()))?;
    serde_json::from_str(&raw).map_err(|e| format!("{} illisible : {e}", path.display()))
}

pub fn list<T: Entity>(app: &AppHandle) -> Result<Vec<T>, String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(entity_dir(app, T::KIND)?).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let id = path.file_stem().unwrap().to_string_lossy().into_owned();
            out.push(load(app, &id)?);
        }
    }
    Ok(out)
}

/// Écrit sans poser de question : sert aussi bien à l'enregistrement demandé
/// par l'utilisateur qu'à l'installation du catalogue de base, qui doit
/// pouvoir écraser un fichier existant.
fn write<T: Entity>(app: &AppHandle, entity: &T) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(entity).map_err(|e| e.to_string())?;
    fs::write(entity_path(app, T::KIND, entity.id())?, raw).map_err(|e| e.to_string())
}

pub fn save<T: Entity>(app: &AppHandle, entity: &T) -> Result<(), String> {
    refuse_if_builtin(T::KIND, entity.id())?;
    write(app, entity)
}

pub fn delete<T: Entity>(app: &AppHandle, id: &str) -> Result<(), String> {
    refuse_if_builtin(T::KIND, id)?;
    remove(app, T::KIND, id)
}

/// Retire un élément du disque, sans se demander à qui il appartient : les
/// appelants ont déjà tranché.
fn remove(app: &AppHandle, kind: EntityKind, id: &str) -> Result<(), String> {
    let path = entity_path(app, kind, id)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(crate::seed::default_settings());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| format!("réglages illisibles : {e}"))
}

pub fn save_settings(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let raw = serde_json::to_string_pretty(s).map_err(|e| e.to_string())?;
    fs::write(settings_path(app)?, raw).map_err(|e| e.to_string())
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
    Ok(data_dir(app)?.join("builtins.json"))
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
        return Err(format!(
            "{} fait partie du catalogue livré avec le logiciel : elle ne peut être ni modifiée ni supprimée ici. Duplique-la pour partir de sa configuration.",
            kind.this()
        ));
    }
    Ok(())
}

/// Ce qu'une version précédente avait installé et que celle-ci ne livre plus.
///
/// C'est la seule liste que la synchronisation a le droit de supprimer. Elle se
/// calcule par différence contre le manifeste, jamais contre le contenu du
/// dossier : un élément absent du manifeste a été créé par l'utilisateur, et il
/// n'appartient pas au logiciel de l'effacer.
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
        write(app, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_bumpers() {
        write(app, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_bumper_bars() {
        write(app, &builtin.model)?;
    }
    for builtin in crate::seed::builtin_clusters() {
        write(app, &builtin.model)?;
    }

    for (kind, id) in stale_builtins(&previous, &current) {
        remove(app, kind, &id)?;
    }

    let raw = serde_json::to_string_pretty(&current).map_err(|e| e.to_string())?;
    fs::write(builtins_manifest_path(app)?, raw).map_err(|e| e.to_string())
}

/// Peuple le répertoire de données au premier lancement, puis installe et met à
/// jour le catalogue de base.
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

    /// Le perçage de couronne est déclaré par l'enceinte, plus déduit d'une
    /// parité. Ce qu'il faut vérifier n'est donc plus « chaque trou est-il du
    /// bon côté de la parité », mais « la fiche livrée déclare-t-elle bien le
    /// perçage relevé » : c'est elle qui fait foi au calcul.
    #[test]
    fn every_shipped_speaker_declares_the_arc_each_hole_is_drilled_on() {
        use sa303_core::speaker::CrownRow;

        let speakers: Vec<_> = crate::seed::builtin_speakers()
            .into_iter()
            .map(|b| b.model)
            .collect();
        assert!(!speakers.is_empty(), "aucune enceinte livrée");

        for model in &speakers {
            let id = &model.id;
            let crown = &model.mechanical.crown;

            for &s in &crate::seed::SA303_SHORT_ARC_SPLAYS {
                assert_eq!(crown.row_at(s), CrownRow::Int, "{id} : {s}° hors arc court");
            }
            for &s in &crate::seed::SA303_LONG_ARC_SPLAYS {
                assert_eq!(crown.row_at(s), CrownRow::Ext, "{id} : {s}° hors arc long");
            }
            // Et la déclaration ne contient rien d'autre que l'arc court : un
            // splay en trop y serait silencieusement mis sur le petit rayon.
            assert_eq!(
                crown.inner_splays.len(),
                crate::seed::SA303_SHORT_ARC_SPLAYS.len(),
                "{id} : arc court déclaré différent du perçage relevé"
            );
            // Les deux arcs réunis sont exactement la grille de splay percée :
            // un trou absent des deux n'aurait aucune rangée déclarée.
            let mut declared = model.mechanical.splay_grid.clone();
            declared.sort_by(f64::total_cmp);
            assert_eq!(declared, crate::seed::sa303_splay_grid(), "{id} : grille");
        }
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
        // travail de l'utilisateur.
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
}

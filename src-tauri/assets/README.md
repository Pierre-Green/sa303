# Catalogue de base

Un dossier par famille (`speakers`, `bumpers`, `bumper-bars`, `clusters`), un
fichier JSON par élément, au format exact de ceux qu'écrit l'application dans
son dossier de données.

Ces éléments sont **immuables côté application** : l'interface les affiche en
lecture seule et la persistance refuse de les enregistrer ou de les supprimer.
Ils ne changent que par une mise à jour du logiciel.

## Comment ça se propage

À chaque lancement, `persistence::sync_builtins` réécrit tous ces fichiers dans
le dossier de données. Une mise à jour corrige donc une installation existante
sans qu'il faille effacer ce dossier.

Un `builtins.json` est tenu à jour à côté des catalogues : il liste ce que la
version installée a livré. C'est lui qui permet de retirer un élément qu'une
mise à jour ne livre plus, **et seulement celui-là** — tout ce que
l'utilisateur a créé lui est inconnu, donc intouchable.

Les réglages (gravité, coefficient dynamique, ...) ne sont pas dans ce
catalogue : l'utilisateur les ajuste, ils ne sont créés que s'ils manquent.

## Comment en ajouter un

Le composer dans l'application, puis copier son fichier depuis le dossier de
données vers le dossier de la bonne famille. Rien d'autre : `build.rs` recense
le contenu des dossiers, il n'y a pas de liste à tenir à jour.

Le nom du fichier doit être l'identifiant de l'élément, sans quoi il serait
introuvable par son propre `id` — un test le vérifie.

## Ce que les tests garantissent

- chaque asset se désérialise (`every_asset_is_named_after_the_id_it_declares`) ;
- chaque grappe livrée passe le solveur, donc ses angles sont percés et ses
  jonctions déclarées compatibles ;
- le perçage des enceintes livrées correspond au perçage de référence ;
- une mise à jour ne supprime que ce qu'une version précédente avait installé.

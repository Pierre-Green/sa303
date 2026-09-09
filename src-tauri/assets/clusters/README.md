# Grappes d'exemple

Un fichier JSON par grappe, au format exact de ceux qu'écrit l'application dans
son dossier de données. Ils sont embarqués dans le binaire à la compilation et
enregistrés au premier lancement, quand le dossier des grappes est encore vide.

Pour en ajouter une : la composer dans l'éditeur, puis copier son fichier depuis
le dossier de données de l'application vers ce dossier. Rien d'autre à faire —
`build.rs` recense le contenu du dossier, il n'y a pas de liste à tenir à jour.

Le nom du fichier n'a pas d'importance : c'est le champ `id` qui sert de nom au
fichier une fois enregistré. Deux exemples ne doivent pas partager le même `id`,
sinon le second écrase le premier.

Chaque fichier est relu par les tests (`every_seeded_cluster_asset_describes_a_buildable_cluster`) :
un exemple dont un angle n'est pas percé, dont une jonction n'est pas déclarée
compatible ou dont le bumper ne convient pas fait échouer la compilation des
tests plutôt que d'être livré cassé.

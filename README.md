# Squat Souls

Petite appli desktop (Rust, Tauri 2, Vue 3) qui tire un exercice au poids du corps et un nombre de répétitions au hasard quand un événement arrive en jeu (mort, repos au feu, boss vaincu...).

## Installation (Arch Linux)

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg
# Rust et Node si besoin
sudo pacman -S --needed rustup nodejs pnpm && rustup default stable

cd SquatSouls
pnpm install
pnpm tauri dev        # lancement en développement
pnpm tauri build      # binaire + paquets dans src-tauri/target/release/bundle/
```

Le binaire seul est dans `src-tauri/target/release/squat-souls` après le build.

Si la fenêtre reste blanche sous Wayland (cas fréquent avec un GPU NVIDIA) :

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 pnpm tauri dev
```

## Utilisation

- Choisis le jeu, puis clique sur l'événement qui vient d'arriver, ou tape son numéro (1 à 9, rangée du haut ou pavé numérique, sans Maj sur AZERTY).
- La série tirée apparaît dans « À faire ». **Entrée** (ou Espace) valide la plus ancienne, **Retour arrière** l'annule, **J** la saute avec un joker.
- « Terminer la séance » et « Annuler la séance », en bas de l'onglet Jouer, demandent une confirmation.
- L'onglet **Bilan** montre la séance actuelle et les 7 derniers jours. **Tout voir** affiche tout l'historique en grand (Échap pour fermer).

## Règles métier

Cette section fait référence : toute évolution du comportement de l'appli doit y être reportée.

### Validation des actions

- Une action ne se valide qu'une fois : le bouton qui la valide (**Créer**, confirmation d'une suppression, **Terminer**, etc.) l'applique et l'enregistre aussitôt. Aucune action déjà validée ne doit attendre une seconde validation, comme le bouton **Enregistrer** de la configuration.
- **Enregistrer** ne sert qu'aux modifications faites directement dans les champs, qui n'ont pas de bouton de validation propre.

### Tirage

- Chaque jeu propose une liste d'événements. Déclencher un événement tire `draws` exercices distincts parmi tous les exercices définis, chacun avec un nombre entier pris au hasard entre son `min` et son `max` (inclus).
- L'intervalle appartient à l'exercice : il est le même quel que soit l'événement.
- Un tirage devient une série « à faire ». Les séries à faire se traitent de la plus ancienne à la plus récente.

### Limite par séance

- Chaque déclenchement d'un événement est compté pour la séance, par jeu et par événement.
- Au-delà de `maxPerSession` déclenchements, l'événement ne donne plus de série : un message indique que la limite est atteinte. Le compteur de déclenchements reste alors bloqué à la limite.

### Traitement d'une série à faire

- **Valider** : la série passe dans les séries faites, est ajoutée aux totaux et écrite dans l'historique. Les jokers de l'événement sont gagnés à ce moment.
- **Annuler** (clic raté) : la série disparaît et le déclenchement est décompté, comme s'il n'avait jamais eu lieu.
- **Sauter avec un joker** : possible seulement s'il reste un joker. La série n'est pas faite (ni totaux, ni historique) mais le déclenchement reste compté, y compris pour la limite par séance. Elle apparaît dans les séries sautées du Bilan.

### Jokers

- Seuls les événements avec `jokers` en rapportent, une fois leur série validée.
- Les jokers non dépensés sont perdus à la fin de la séance.

### Séance

- Une séance commence à l'ouverture de l'appli et après chaque fin ou annulation de séance.
- Le jeu est fixé au premier événement déclenché : il ne peut plus changer avant la séance suivante. Le dernier jeu choisi est retenu d'un lancement à l'autre. Au tout premier lancement, ou si ce jeu n'existe plus, aucun jeu n'est sélectionné : il faut le choisir avant de déclencher un événement.
- **Terminer la séance** n'est possible qu'avec au moins une série faite et aucune série à faire. L'heure de fin est enregistrée dans l'historique et une nouvelle séance commence aussitôt.
- **Annuler la séance** remet tout à zéro (compteurs, jokers, séries à faire et faites) et retire de l'historique les séries de cette séance. Le bouton reste désactivé tant qu'aucun événement n'a été déclenché.
- Une configuration enregistrée s'applique immédiatement, sans redémarrer. Elle retire les séries à faire de la séance en cours, sans toucher aux séries faites ni aux compteurs (renommer un jeu ou un événement garde ses compteurs). Si elle supprime le jeu de la séance en cours, la séance est annulée (comme avec **Annuler la séance**) et le jeu est à choisir de nouveau.

### Historique et bilan

- Chaque série validée est écrite dans l'historique (une entrée par exercice), avec l'heure, la séance, le jeu, l'événement, l'exercice et la quantité. Il est conservé d'un lancement à l'autre.
- Le bilan regroupe les séries par jour local, puis par séance et par jeu, avec les totaux par exercice (quantité cumulée et nombre de séries).
- La durée d'une séance va de son début à son heure de fin si elle a été terminée, sinon à sa dernière série.

### Configuration

- La configuration est validée avant d'être enregistrée. Elle est refusée si : il n'y a aucun exercice ou aucun jeu, un id ou un nom est vide, un id est en double, deux jeux portent le même nom (sans tenir compte de la casse ni des espaces autour), un jeu n'a aucun événement, `draws` vaut 0 ou dépasse le nombre d'exercices définis, `maxPerSession` vaut 0, ou un exercice a `max` à 0 ou `min` supérieur à `max`.
- Les jeux sont toujours listés par nom, sans tenir compte de la casse ni des accents, dans l'onglet Jouer comme dans la configuration. Sans jeu choisi, la configuration s'ouvre sur le premier de la liste.
- Une base neuve (premier lancement, aucun jeu enregistré) reçoit la configuration par défaut, décrite dans `src-tauri/src/fitness/seed.json`. Si la configuration enregistrée est illisible ou invalide, la configuration par défaut est utilisée et l'erreur est affichée.

### Partage

- Un jeu se partage seul : son nom et ses événements (nombre tiré, limite, jokers), sans les exercices, propres à chacun. Les exercices se partagent à part : nom, unité, minimum et maximum.
- **Partager** copie dans le presse-papiers un texte à transmettre, tiré de la configuration enregistrée (les modifications en cours n'y sont pas).
- **Importer** demande de coller ce texte et n'est possible que s'il est valide. L'import s'enregistre aussitôt, sans passer par **Enregistrer**, et garde les autres modifications en cours.
- Un jeu importé s'ajoute, ou remplace le jeu qui porte le même nom (sans tenir compte de la casse ni des espaces autour) : le formulaire l'annonce avant l'import. Un événement qui tire plus d'exercices qu'il n'en est défini est ramené à ce nombre, et le message de confirmation le signale.
- Des exercices importés mettent à jour l'unité, le minimum et le maximum de ceux qui portent le même nom et ajoutent les autres ; aucun exercice n'est retiré.

## Configuration

L'onglet **Configuration** se règle avec des champs :

- **Exercices** : nom, unité, minimum et maximum de chaque exercice. Le dernier exercice ne peut pas être supprimé. **Partager** et **Importer** échangent la liste avec d'autres utilisateurs.
- **Jeux** : choisir le jeu à modifier dans la liste (qui affiche les noms enregistrés, par ordre alphabétique), le renommer, le supprimer (après confirmation) ou en ajouter un. Un nouveau jeu demande un nom qu'aucun autre jeu ne porte et reprend soit les événements d'un autre jeu, soit un seul événement à remplir. **Partager** copie le jeu ouvert, **Importer un jeu** en ajoute ou en remplace un à partir d'un texte partagé. Le bouton **Créer**, l'import et la confirmation de suppression enregistrent aussitôt, sans passer par **Enregistrer** ; les autres modifications en cours restent à enregistrer.
- **Événements** du jeu : repliés sur une ligne qui résume le nombre tiré, la limite et les jokers, ils se déplient d'un clic pour régler le nom, le nombre d'exercices tirés, la limite par séance (vide pour aucune) et les jokers gagnés.

Dès qu'une modification est en cours, deux boutons apparaissent : **Enregistrer** valide la configuration et l'applique aussitôt à l'onglet Jouer, **Réinitialiser les modifications** revient à la dernière version enregistrée.

Tout est enregistré dans une base SQLite, `~/.config/com.semche.squatsouls/squatsouls.db` : configuration, historique et dernier jeu choisi. À l'enregistrement, seuls les exercices, jeux et événements modifiés sont réécrits.

## Tests

```bash
cd src-tauri && cargo test # backend
pnpm test                   # interface (Vitest)
```

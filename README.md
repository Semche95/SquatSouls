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

### Tirage

- Chaque jeu propose une liste d'événements. Déclencher un événement tire `draws` exercices distincts parmi tous les exercices définis, chacun avec un nombre entier pris au hasard entre son `min` et son `max` (inclus).
- L'intervalle appartient à l'exercice : il est le même quel que soit l'événement.
- Un tirage devient une série « à faire ». Les séries à faire se traitent de la plus ancienne à la plus récente.
- Exercices par défaut : pompes 8 à 12, tractions 2 à 4, squats 12 à 18, fentes 5 à 8 par jambe, gainage 20 à 45 s, superman 10 à 15.
- Événements par défaut, communs à tous les Souls (seul le vocabulaire change selon le jeu) :

| Événement | Tirés | Particularité |
|---|---|---|
| Mort | 1 | 15 au plus par séance |
| Repos à un feu connu | 1 | |
| Nouveau feu allumé | 1 | |
| Montée de niveau | 1 | |
| Âmes perdues | 1 | |
| Boss vaincu | 2 | |
| Boss vaincu au premier essai | 2 | rapporte 1 joker |
| Boss vaincu sans prendre un coup | 2 | rapporte 2 jokers |

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
- Une base neuve reçoit la configuration par défaut. Si la configuration enregistrée est illisible ou invalide, la configuration par défaut est utilisée et l'erreur est affichée.

## Configuration

L'onglet **Configuration** se règle avec des champs :

- **Exercices** : nom, unité, minimum et maximum de chaque exercice. Le dernier exercice ne peut pas être supprimé.
- **Jeux** : choisir le jeu à modifier dans la liste, le renommer, le supprimer ou en ajouter un. Un nouveau jeu demande un nom qu'aucun autre jeu ne porte et reprend soit les événements d'un autre jeu, soit un seul événement à remplir.
- **Événements** du jeu : repliés sur une ligne qui résume le nombre tiré, la limite et les jokers, ils se déplient d'un clic pour régler le nom, le nombre d'exercices tirés, la limite par séance (vide pour aucune) et les jokers gagnés.

Dès qu'une modification est en cours, deux boutons apparaissent : **Enregistrer** valide la configuration et l'applique aussitôt à l'onglet Jouer, **Réinitialiser les modifications** revient à la dernière version enregistrée.

Tout est enregistré dans une base SQLite, `~/.config/com.semche.squatsouls/squatsouls.db` : configuration, historique et dernier jeu choisi. À l'enregistrement, seuls les exercices, jeux et événements modifiés sont réécrits.

## Tests

```bash
cd src-tauri && cargo test # backend
pnpm test                   # interface (Vitest)
```

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

Le workflow GitHub Actions **Build** (`.github/workflows/build.yml`) compile les paquets Linux (`.deb`, `.rpm`, `.AppImage`) et Windows (`.msi`, `.exe`). Il se lance à la main depuis l'onglet Actions (**Run workflow**), et les paquets s'y téléchargent dans les artefacts de l'exécution.

Sous Linux, l'appli force `GDK_BACKEND=x11` et `WEBKIT_DISABLE_DMABUF_RENDERER=1` au lancement : WebKitGTK plante ou reste blanc sous Wayland (presse-papiers, GPU NVIDIA), elle passe donc par XWayland sans rendu DMABUF.

## Utilisation

- Choisis le jeu, puis clique sur l'événement qui vient d'arriver, ou tape son numéro (1 à 9, rangée du haut ou pavé numérique, sans Maj sur AZERTY).
- La série tirée apparaît dans « À faire ». Chaque exercice se note fait avec son bouton **Fait**. Au clavier, sur la plus ancienne série, **Entrée** (ou Espace) note fait son premier exercice restant, **Retour arrière** annule la série (après confirmation si elle est commencée). Avec des jokers, **H** réduit de moitié ce même exercice et **J** le saute.
- Après chaque exercice fait, une pause s'affiche avec son compte à rebours : l'exercice suivant ne se note fait qu'à sa fin. **Passer** (ou Échap) l'écourte.
- « Terminer la séance » et « Annuler la séance », en bas de l'onglet Jouer, demandent une confirmation.
- L'onglet **Bilan** montre la séance actuelle et les 7 derniers jours. **Tout voir** affiche tout l'historique en grand (Échap pour fermer).
- Avec **Détecter le jeu Steam en cours** (Configuration › Réglages), l'onglet Jouer propose de sélectionner le jeu Steam lancé, ou de l'ajouter s'il n'est pas encore configuré, et une notification système signale chaque jeu lancé.
- Avec **Afficher l'icône dans la zone de notification** (Configuration › Réglages), l'appli peut rester ouverte fenêtre masquée.

## Règles métier

Cette section fait référence : toute évolution du comportement de l'appli doit y être reportée.

### Validation des actions

- Une action ne se valide qu'une fois : le bouton qui la valide (**Créer**, confirmation d'une suppression, **Terminer**, etc.) l'applique et l'enregistre aussitôt. Aucune action déjà validée ne doit attendre une seconde validation, comme le bouton **Enregistrer** de la configuration.
- **Enregistrer** ne sert qu'aux modifications faites directement dans les champs, qui n'ont pas de bouton de validation propre.

### Tirage

- Chaque jeu propose une liste d'événements. Déclencher un événement tire `draws` exercices distincts parmi tous les exercices définis, chacun avec un nombre entier pris au hasard entre son `min` et son `max` (inclus).
- Les exercices sortent par tours : dans une séance, chaque exercice sort une fois avant que l'un d'eux puisse revenir. Quand un tour est épuisé, un nouveau commence ; il ne reprend aussitôt ni un exercice de la série précédente, ni un exercice déjà sorti dans le même tirage (sauf s'il ne reste que lui à tirer pour le premier cas). Une configuration enregistrée recommence le tour.
- L'intervalle appartient à l'exercice : il est le même quel que soit l'événement.
- Un tirage devient une série « à faire ». Les séries à faire se traitent de la plus ancienne à la plus récente.

### Limite par séance

- Chaque déclenchement d'un événement est compté pour la séance, par jeu et par événement.
- Au-delà de `maxPerSession` déclenchements, l'événement ne donne plus de série : un message indique que la limite est atteinte. Le compteur de déclenchements reste alors bloqué à la limite.

### Traitement d'une série à faire

- **Fait** se note exercice par exercice : l'exercice est ajouté aux totaux et écrit dans l'historique, et reste affiché barré jusqu'à la fin de la série. La série passe dans les séries faites quand son dernier exercice est fait ou sauté, et les jokers de l'événement sont gagnés à ce moment.
- **Annuler** (clic raté) : la série disparaît et le déclenchement est décompté, comme s'il n'avait jamais eu lieu. Rien n'en est compté : ses exercices déjà faits sont retirés des totaux et de l'historique, pour qu'annuler ne serve pas à écourter une série. Une série commencée ne s'annule donc qu'après confirmation. La série annulée est réservée : le prochain déclenchement du même événement redonne les mêmes exercices avec les mêmes nombres, tels que tirés, pour qu'annuler ne serve pas à changer de tirage. La réserve disparaît avec la séance (fin ou annulation) ; une configuration enregistrée l'ajuste comme les séries à faire.
- **Jokers** : réduire de moitié ou sauter un exercice, ou sauter toute la série (voir Jokers). Le déclenchement reste compté, y compris pour la limite par séance.

### Pause entre les exercices

- Chaque exercice fait (**Fait**) lance une pause : sa pause propre, ou à défaut la pause globale (60 s dans la configuration par défaut). Une pause de 0 s n'en lance pas.
- Pendant la pause, aucun exercice ne peut être noté fait, ni dans la même série ni dans une autre, y compris une série tirée pendant la pause. Les jokers et **Annuler** restent possibles et ne lancent pas de pause.
- **Passer** (ou Échap) termine la pause tout de suite. Elle s'arrête aussi à l'annulation de la série dont un exercice l'a lancée, et à la fin ou à l'annulation de la séance.

### Jokers

- Seuls les événements avec `jokers` en rapportent, une fois tous les exercices de leur série faits : dépenser un joker sur une série (moitié ou saut) lui fait perdre ceux qu'elle devait rapporter.
- **Moitié** (1 joker) : divise le nombre d'un exercice par deux, arrondi au supérieur. Une seule fois par exercice, et pas pour un nombre de 1.
- **Sauter** un exercice (2 jokers, 1 seul s'il a déjà été réduit de moitié) : il est retiré de la série, le reste demeure à faire. Sauter le dernier exercice retire la série.
- **Sauter la série** (série à plusieurs exercices restants) : coûte la somme des sauts de ses exercices restants ; ceux déjà faits restent faits.
- Un exercice fait ne peut plus être réduit ni sauté.
- Une action n'est possible qu'avec assez de jokers. Ce qui est retiré n'est pas fait (ni totaux, ni historique) et apparaît dans les jokers du Bilan, marqué « moitié » ou « sauté ».
- Les jokers non dépensés sont perdus à la fin de la séance.

### Séance

- Une séance commence à l'ouverture de l'appli et après chaque fin ou annulation de séance.
- Le jeu est fixé au premier événement déclenché : il ne peut plus changer avant la séance suivante. Le dernier jeu choisi est retenu d'un lancement à l'autre. Au tout premier lancement, ou si ce jeu n'existe plus, aucun jeu n'est sélectionné : il faut le choisir avant de déclencher un événement.
- **Terminer la séance** n'est possible qu'avec au moins une série faite et aucune série à faire. L'heure de fin est enregistrée dans l'historique et une nouvelle séance commence aussitôt.
- **Annuler la séance** remet tout à zéro (compteurs, jokers, séries à faire et faites) et retire de l'historique les séries de cette séance. Le bouton reste désactivé tant qu'aucun événement n'a été déclenché.
- Une configuration enregistrée s'applique immédiatement, sans redémarrer. Elle ne touche ni aux séries faites ni aux compteurs (renommer un jeu ou un événement garde ses compteurs). Les séries à faire restent telles que tirées, avec les noms à jour, sauf :
  - un événement supprimé retire ses séries à faire ;
  - un nombre d'exercices tirés (`draws`) changé ajuste ses séries : les exercices manquants sont tirés, ou des exercices pas encore faits sont retirés ; une série où il ne reste rien à faire est finie et rapporte ses jokers ;
  - un exercice supprimé est remplacé par un autre tiré au hasard, absent de la série ;
  - un minimum ou un maximum changé ramène le nombre d'un exercice à faire dans les nouvelles bornes, sauf s'il a été réduit de moitié ;
  - des jokers changés s'appliquent aux séries de l'événement, sauf à celles où des jokers ont déjà été dépensés, qui n'en rapportent plus.

  Les exercices déjà faits ne changent jamais. Si elle supprime le jeu de la séance en cours, la séance est annulée (comme avec **Annuler la séance**) et le jeu est à choisir de nouveau.

### Historique et bilan

- Chaque exercice fait est écrit dans l'historique (une entrée par exercice), avec l'heure, la séance, le jeu, l'événement, l'exercice et la quantité. Il est conservé d'un lancement à l'autre.
- Le bilan regroupe les séries par jour local, puis par séance et par jeu, avec les totaux par exercice (quantité cumulée et nombre de séries).
- La durée d'une séance va de son début à son heure de fin si elle a été terminée, sinon à sa dernière série.

### Configuration

- La configuration est validée avant d'être enregistrée. Elle est refusée si : il n'y a aucun exercice ou aucun jeu, un id ou un nom est vide, un id est en double, deux jeux portent le même nom (sans tenir compte de la casse ni des espaces autour), un jeu n'a aucun événement, `draws` vaut 0 ou dépasse le nombre d'exercices définis, `maxPerSession` vaut 0, ou un exercice a `max` à 0 ou `min` supérieur à `max`.
- Chaque champ numérique (pauses, minimum, maximum, exercices tirés, limite, jokers) doit être un nombre entier, 0 ou plus, et pas trop grand. Ce format se vérifie pendant la saisie : un champ mal rempli (texte, nombre négatif ou décimal) s'encadre en rouge et le problème s'affiche aussitôt dans une infobulle pendant la saisie ; une fois le champ quitté, l'infobulle se ferme et se rouvre à son survol. Les autres erreurs et les confirmations n'apparaissent qu'à l'enregistrement. Tant qu'un champ est mal rempli, **Enregistrer** est désactivé et son infobulle nomme le premier champ fautif (par exemple « Squats › minimum : il faut un nombre entier, 0 ou plus. »). Un champ obligatoire vidé vaut 0, un champ facultatif vidé (pause propre, limite) est retiré.
- Les jeux sont toujours listés par nom, sans tenir compte de la casse ni des accents, dans l'onglet Jouer comme dans la configuration. Sans jeu choisi, la configuration s'ouvre sur le premier de la liste.
- Une base neuve (premier lancement, aucun jeu enregistré) reçoit la configuration par défaut, décrite dans `src-tauri/src/fitness/seed.json`. Si la configuration enregistrée est illisible ou invalide, la configuration par défaut est utilisée et l'erreur est affichée.

### Détection des jeux Steam

- Facultative, désactivée par défaut : la case **Détecter le jeu Steam en cours** des Réglages fait partie de la configuration et s'enregistre avec **Enregistrer**.
- Activée, l'appli lit les processus toutes les 10 secondes et y cherche l'AppId d'un jeu lancé par Steam (`SteamLaunch AppId=…` dans sa ligne de commande, ou `SteamAppId` dans son environnement). Sans résultat et si un processus Steam tourne, elle prend l'AppId que Steam note pour le jeu en cours (`RunningAppID` du registre sous Windows, de `registry.vdf` sous macOS et Linux), pour les jeux dont l'environnement est illisible. Seuls les jeux Steam sont détectés, et aucune liste de jeux n'est embarquée.
- Un jeu configuré se reconnaît à son AppId Steam. Plusieurs jeux peuvent porter le même : un jeu de la configuration est un réglage de défi, et un même jeu Steam peut en avoir plusieurs. Un jeu sans AppId se reconnaît à son nom, qu'il reprend ou prolonge sans y coller de lettre (« Megabonk 2 » pour « Megabonk »), sans tenir compte de la casse ni des espaces autour : le nom des jeux qui portent l'AppId détecté, ou à défaut celui donné par le Steam Store. Les jeux de la configuration par défaut ont leur AppId, sauf Bloodborne, absent de Steam.
- Jeu configuré : l'onglet Jouer propose un bouton par jeu reconnu pour le sélectionner, sauf si l'un d'eux l'est déjà ou si le jeu de la séance est fixé. Reconnu à son nom, il reçoit alors l'AppId détecté, enregistré aussitôt.
- Jeu inconnu : son nom est demandé à l'API du Steam Store (sans les marques ™ et ®), puis l'onglet Jouer propose de l'**Ajouter** ou de l'**Ignorer**. **Ajouter** ouvre la création d'un jeu, préremplie avec ce nom et l'AppId, qui s'enregistre avec **Créer**. **Ignorer** l'ajoute aussitôt aux jeux ignorés, qui ne sont plus jamais proposés.
- Les Réglages listent les jeux ignorés : **Ajouter** ouvre la même création, et le jeu créé quitte la liste ; **Retirer** l'enlève aussitôt de la liste, et il sera de nouveau proposé.
- Pour ne jamais être bloqué par Steam, le Steam Store n'est appelé que pour un jeu inconnu et non ignoré : un nom obtenu n'est plus jamais redemandé ; un jeu qu'il n'a pas nommé, ou une demande restée sans réponse, est redemandé une heure plus tard au plus tôt ; au plus un appel par minute, avec 10 s maximum d'attente. S'il refuse pour excès de requêtes (code 429 ou 403), il n'est plus appelé pendant 24 h : les jeux inconnus ne sont plus proposés, les jeux configurés restent détectés. Ces délais et les noms obtenus sont gardés dans la base, d'un lancement à l'autre.
- **Notifier quand un jeu est lancé**, visible sous la case de détection cochée et cochée par défaut, envoie une notification système une seule fois par lancement : pour un jeu configuré (« à sélectionner dans Jouer ») ou inconnu (« l'ajouter depuis Jouer ») une fois son nom connu, jamais pour un jeu ignoré. Un jeu relancé après qu'aucun jeu n'a tourné est de nouveau notifié. Elle s'enregistre avec **Enregistrer**.
- Sous la case, les Réglages montrent l'état de la détection enregistrée : le jeu détecté (même déjà sélectionné ou ignoré), aucun jeu, ou un Steam Store qui limite les requêtes. Si les processus sont illisibles, rien n'est détecté ; le passage suivant réessaie.

### Zone de notification

- Facultative, désactivée par défaut : la case **Afficher l'icône dans la zone de notification** des Réglages fait partie de la configuration et s'enregistre avec **Enregistrer**, qui ajoute ou retire aussitôt l'icône.
- Cochée, elle déplie deux cases, décochées par défaut : **Fermer la fenêtre la masque au lieu de quitter** (l'appli et la détection Steam continuent) et **Démarrer masqué** (l'appli démarre sans fenêtre). Sans l'icône, elles sont sans effet : fermer la fenêtre quitte l'appli, qui démarre toujours fenêtre affichée.
- Un clic sur l'icône affiche la fenêtre ; son menu propose **Afficher** et **Quitter** (sous Linux, seul le menu est disponible).

### Partage

- Un jeu se partage seul : son nom, son AppId Steam s'il en a un, et ses événements (nombre tiré, limite, jokers), sans les exercices, propres à chacun. Les exercices se partagent à part : nom, unité, minimum, maximum et pause propre. La pause globale n'est pas partagée.
- **Partager** copie dans le presse-papiers un texte compressé à transmettre (illisible tel quel, il résiste aux retours à la ligne ajoutés par les messageries), tiré de la configuration enregistrée (les modifications en cours n'y sont pas). Il commence par un court identifiant qui indique le contenu : `squatsouls/game/elden-ring:` ou `squatsouls/exercises:`. Cet identifiant n'est qu'indicatif : l'import l'ignore et lit la version du format et le type de contenu dans le texte compressé. L'import lit chaque texte selon sa version et refuse, en demandant de mettre à jour l'appli, une version qu'il ne connaît pas.
- **Importer** demande de coller ce texte et n'est possible que s'il est valide. Un jeu collé dans l'import des exercices, ou l'inverse, est refusé avec un message qui indique où l'importer. L'import s'enregistre aussitôt, sans passer par **Enregistrer**, et garde les autres modifications en cours.
- Un jeu importé s'ajoute, ou remplace le jeu qui porte le même nom (sans tenir compte de la casse ni des espaces autour) : le formulaire l'annonce avant l'import. Sans AppId Steam dans l'import, le jeu remplacé garde le sien. Un événement qui tire plus d'exercices qu'il n'en est défini est ramené à ce nombre, et le message de confirmation le signale.
- Des exercices importés mettent à jour l'unité, le minimum, le maximum et la pause propre de ceux qui portent le même nom (sans pause propre dans l'import, l'exercice repasse à la pause globale) et ajoutent les autres ; aucun exercice n'est retiré.

## Configuration

L'onglet **Configuration** se règle avec des champs :

- **Exercices** : la pause globale après chaque série, puis le nom, l'unité, le minimum, le maximum et la pause propre (vide pour suivre la pause globale) de chaque exercice. Le dernier exercice ne peut pas être supprimé. **Partager** et **Importer** échangent la liste avec d'autres utilisateurs.
- **Jeux** : choisir le jeu à modifier dans la liste (qui affiche les noms enregistrés, par ordre alphabétique), le renommer, le supprimer (après confirmation) ou en ajouter un. Un nouveau jeu demande un nom qu'aucun autre jeu ne porte et reprend soit les événements d'un autre jeu, soit un seul événement à remplir. **Partager** copie le jeu ouvert, **Importer un jeu** en ajoute ou en remplace un à partir d'un texte partagé. Le bouton **Créer**, l'import et la confirmation de suppression enregistrent aussitôt, sans passer par **Enregistrer** ; les autres modifications en cours restent à enregistrer.
- **Événements** du jeu : repliés sur une ligne qui résume le nombre tiré, la limite et les jokers, ils se déplient d'un clic pour régler le nom, le nombre d'exercices tirés, la limite par séance (vide pour aucune) et les jokers gagnés.
- **Réglages** : l'icône dans la zone de notification (voir Zone de notification), la détection du jeu Steam en cours et sa notification, et la liste des jeux ignorés (voir Détection des jeux Steam).

Dès qu'une modification est en cours, deux boutons apparaissent : **Enregistrer** valide la configuration et l'applique aussitôt à l'onglet Jouer, **Réinitialiser les modifications** revient à la dernière version enregistrée.

Tout est enregistré dans une base SQLite, `~/.config/com.semche.squatsouls/squatsouls.db` (`squatsouls-dev.db` pour l'appli de développement, lancée par `pnpm tauri dev`) : configuration (dont la pause globale, la zone de notification, la détection Steam, sa notification et les jeux ignorés), historique, dernier jeu choisi et noms donnés par le Steam Store. À l'enregistrement, seuls les exercices, jeux et événements modifiés sont réécrits. Au lancement, la base reçoit les migrations de schéma qui lui manquent, chacune datée à son application (une migration créée ne se modifie jamais, toute évolution du schéma en ajoute une nouvelle) ; une base créée par une version plus récente de l'appli est refusée.

## Tests

```bash
cd src-tauri && cargo test # backend
pnpm test                   # interface (Vitest)
```

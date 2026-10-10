# Souls Fit

Petite appli desktop (Rust, Tauri 2, Vue 3) qui tire un exercice au poids du corps et un nombre de répétitions au hasard quand un événement arrive en jeu (mort, repos au feu, boss vaincu...).

## Installation (Arch Linux)

```bash
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl appmenu-gtk-module libappindicator-gtk3 librsvg
# Rust et Node si besoin
sudo pacman -S --needed rustup nodejs npm && rustup default stable

cd souls-fit
npm install
npm run tauri dev        # lancement en développement
npm run tauri build      # binaire + paquets dans src-tauri/target/release/bundle/
```

Le binaire seul est dans `src-tauri/target/release/souls-fit` après le build.

Si la fenêtre reste blanche sous Wayland (cas fréquent avec un GPU NVIDIA) :

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 npm run tauri dev
```

## Utilisation

- Choisis le jeu, puis clique sur l'événement qui vient d'arriver, ou tape son numéro (1 à 9, rangée du haut ou pavé numérique, sans Maj sur AZERTY).
- La série tirée apparaît dans « À faire ». **Entrée** (ou Espace) valide la plus ancienne, **Retour arrière** l'annule (clic raté, elle n'est pas comptée).
- L'onglet **Séance** affiche les totaux de la soirée, les séries faites et les 7 derniers jours. « Nouvelle séance » remet les compteurs à zéro (dont la limite de morts).

## Configuration

L'onglet **Configuration** édite directement le fichier JSON (validé à l'enregistrement) :

- `~/.config/com.semche.soulsfit/config.json` : exercices, jeux, événements et intervalles
- `~/.local/share/com.semche.soulsfit/history.jsonl` : une ligne par série faite

Pour chaque événement :

| Champ | Rôle |
|---|---|
| `exercises` | exercices possibles, chacun avec son `min` et son `max` (inclus) |
| `draws` | nombre d'exercices différents tirés (3 pour un boss vaincu, 1 par défaut) |
| `maxPerSession` | facultatif : au-delà de ce nombre dans la séance, plus de série (15 pour la mort) |

Pour ajouter un jeu, copie le bloc d'un jeu existant dans `games` et change son `id` et son `name`.

## Tests

```bash
cd src-tauri && cargo test
```

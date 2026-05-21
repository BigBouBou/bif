# bif

`bif` est une mini app CLI de prise de notes « lazy » (Before I Forget).

- Les notes sont des **records d’une ligne** ajoutés dans un fichier `.bif`.
- Par défaut, le fichier suivi est `log.bif`.
- Tu peux **créer plusieurs fichiers `.bif`** et choisir lequel est « tracké » (par répertoire).

## Installation

### Prérequis

- Rust toolchain (stable) + Cargo : https://rustup.rs

Vérifie que tout est OK :

```sh
rustc --version
cargo --version
```

### Installer depuis le repo (recommandé)

Le binaire `bif` s’installe via Cargo :

```sh
cargo install --git https://github.com/BigBouBou/bif
```

Puis vérifie :

```sh
bif help
```

Notes :
- Si `bif` n’est pas trouvé, assure-toi que le dossier des binaires Cargo est dans ton `PATH`.
  - Linux/macOS : `~/.cargo/bin`
  - Windows : `%USERPROFILE%\.cargo\bin`
- Pour mettre à jour : relance la commande `cargo install --git ...`.

### Build local (si tu veux contribuer)

```sh
git clone https://github.com/BigBouBou/bif
cd bif
cargo build
```

Lancer sans installer :

```sh
cargo run -- help
```

Installer depuis une copie locale :

```sh
cargo install --path .
```

## Format des notes

Chaque note est écrite sur **une seule ligne**.

Format (legacy, toujours supporté) :

`<stamp>\t<tags>\t<body>`

Format étendu (si des stamps/meta sont configurés) :

`<stamp>\t<tags>\t<body>\t<meta_json>`

- `<stamp>` : format interne `timestamp|LEVEL|SOURCE?` (ex: `1716400000|INFO|`).
- `<tags>` : tags optionnels (CSV, peut être vide).
- `<body>` : texte (échappé pour rester sur une ligne).
- `<meta_json>` : JSON (objet string->string) contenant des stamps calculés à `new` + `_cfg_hash`.

Notes importantes :
- `bif read` (sans `--pretty`) affiche **les lignes brutes** du fichier (script-friendly).
- `bif read --pretty` est une vue **humaine** et peut dépendre de la config globale.

## Utilisation rapide

### Démarrage en 30 secondes

Dans un dossier (par exemple un projet), initialise un log `.bif` et commence à écrire.

- Initialiser (crée `log.bif` dans le dossier courant si besoin) :
  - `bif init`

- Ajouter une note :
  - `bif new "hello"`

- Lire brut (par défaut) :
  - `bif read`

- Lire en joli (vue humaine) :
  - `bif read --pretty`

- Raccourci (capture ultra rapide) :
  - `bif hello`

## Configuration globale (stamps + pretty)

`bif` charge une config **globale** (niveau utilisateur) en JSON.

Chemin par défaut (Unix) :
- `$XDG_CONFIG_HOME/bif/config.json` (si `XDG_CONFIG_HOME` est défini)
- sinon `~/.config/bif/config.json`

Cette config contrôle :
- `new_stamp_ids` : quels “stamp providers” sont exécutés à `bif new` (capture-time) et stockés dans `<meta_json>`
- `pretty.meta_keys` : quels champs de meta afficher dans `bif read --pretty` (view-time), et dans quel ordre

Exemple minimal (par défaut, aucun meta affiché en pretty => fallback sur le stamp legacy) :

```json
{
  "new_stamp_ids": [],
  "pretty": {
    "meta_keys": [],
    "meta_sep": " ",
    "legacy_stamp_format": { "parts": [
      {"Literal": "["},
      "DateYYYY", {"Literal": "-"}, "DateMM", {"Literal": "-"}, "DateDD",
      {"Literal": " "},
      "TimeHH", {"Literal": ":"}, "TimeMM", {"Literal": ":"}, "TimeSS",
      {"Literal": "] "},
      "Level"
    ]}
  }
}
```

Exemple: stocker `time`, `level`, `cwd` à la capture, et afficher `level` + `cwd` en pretty :

```json
{
  "new_stamp_ids": ["time", "level", "cwd"],
  "pretty": {
    "meta_keys": ["level", "cwd"],
    "meta_sep": " ",
    "legacy_stamp_format": { "parts": [] }
  }
}
```

Providers intégrés (IDs disponibles) :
- `time` (timestamp epoch seconds, string)
- `date` (date locale, `YYYY-MM-DD`)
- `datetime` (datetime locale, `YYYY-MM-DDTHH:MM:SS±TZ`)
- `level` (ex: `INFO`)
- `source` (source du stamp si présent, sinon chaîne vide)
- `cwd` (répertoire courant au moment du `new`)

## Plusieurs fichiers `.bif`

Le projet permet de gérer plusieurs logs `.bif` (un par contexte, projet, etc.) **dans un même dossier**.

Commandes :

- Créer un nouveau fichier `.bif` vide dans le dossier courant :
  - `bif init` (crée `log.bif`)
  - `bif init work.bif` (crée `work.bif`)

- Choisir quel fichier est actuellement **tracké** (cible par défaut) :
  - `bif track work.bif`

Le fichier tracké est enregistré dans le dossier courant dans `.bif-tracked`.

> Important : le tracking est **par répertoire**. Si tu `cd` ailleurs, il faudra init/track dans ce nouveau dossier.

## Objectif

Prise de note **instantanée** depuis le terminal, avec un stockage lisible, simple à sauvegarder, aucune friction d’organisation.

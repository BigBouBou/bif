# bif

`bif` est une mini app CLI de prise de notes « lazy ».

- Les notes sont des **records d’une ligne** ajoutés dans un fichier `.bif`.
- Par défaut, le fichier suivi est `log.bif`.
- Tu peux **créer plusieurs fichiers `.bif`** et choisir lequel est « tracké ».

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

- Initialiser (crée `log.bif` si besoin) :
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

Le projet permet de gérer plusieurs logs `.bif` (un par contexte, projet, etc.).

- Créer un nouveau fichier `.bif`
- Choisir quel fichier est actuellement **tracké**

Les commandes exactes peuvent varier selon la version, mais l’idée est toujours la même :

- un seul fichier `.bif` est la cible par défaut
- toutes les captures (`bif ...`) écrivent dans ce fichier

## Objectif

Rendre la prise de note **instantanée** depuis le terminal, avec un stockage lisible, simple à sauvegarder, et sans friction d’organisation.

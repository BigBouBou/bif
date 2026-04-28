# bif

`bif` est une mini app CLI de prise de notes « lazy ».

- Les notes sont des **records d’une ligne** ajoutés dans un fichier `.bif`.
- Par défaut, le fichier suivi est `log.bif`.
- Tu peux **créer plusieurs fichiers `.bif`** et choisir lequel est « tracké ».

## Format des notes

Chaque note est écrite sur **une seule ligne** :

- `<stamp>` : un timestamp (date/heure)
- `<tags>` : tags optionnels
- `<body>` : le texte

Format :

`<stamp>\t<tags>\t<body>`

## Utilisation rapide

- Initialiser (crée `log.bif` si besoin) :
  - `bif init`

- Ajouter une note :
  - `bif new "hello"`

- Raccourci (capture ultra rapide) :
  - `bif hello`

## Plusieurs fichiers `.bif`

Le projet permet de gérer plusieurs logs `.bif` (un par contexte, projet, etc.).

- Créer un nouveau fichier `.bif`
- Choisir quel fichier est actuellement **tracké**

Les commandes exactes peuvent varier selon la version, mais l’idée est toujours la même :

- un seul fichier `.bif` est la cible par défaut
- toutes les captures (`bif ...`) écrivent dans ce fichier

## Objectif

Rendre la prise de note **instantanée** depuis le terminal, avec un stockage lisible, simple à sauvegarder, et sans friction d’organisation.

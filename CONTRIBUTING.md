# Contribuer à Zephyr 🌬️

Tout d'abord, merci de l'intérêt que vous portez à Zephyr ! Ce projet open source vise à fusionner la puissance de Tailwind CSS v4 (via Rust) avec l'écosystème React Native. C'est grâce à la communauté que ce projet peut grandir et s'améliorer.

Ce document détaille le processus pour contribuer au projet, que ce soit pour signaler un bug, proposer une fonctionnalité ou soumettre du code.

## Table des matières

* [Code de Conduite](https://www.google.com/search?q=%23code-de-conduite)
* [Signaler un Bug](https://www.google.com/search?q=%23signaler-un-bug)
* [Proposer une Fonctionnalité](https://www.google.com/search?q=%23proposer-une-fonctionnalit%C3%A9)
* [Architecture du Projet](https://www.google.com/search?q=%23architecture-du-projet)
* [Environnement de Développement](https://www.google.com/search?q=%23environnement-de-d%C3%A9veloppement)
* [Standards de Code](https://www.google.com/search?q=%23standards-de-code)
* [Soumettre une Pull Request (PR)](https://www.google.com/search?q=%23soumettre-une-pull-request-pr)

## Code de Conduite

En participant à ce projet, vous acceptez de maintenir un environnement accueillant, respectueux et harcèlement-free pour tout le monde. Soyez bienveillant et constructif dans vos échanges.

## Signaler un Bug

Si vous rencontrez un problème avec Zephyr, veuillez ouvrir une **Issue** sur GitHub en fournissant :

* Une description claire et concise du problème.
* Les étapes pour reproduire le bug.
* Les versions utilisées (OS, Node.js, Rust, React Native, Zephyr).
* Si possible, un dépôt minimal reproductible (ou un extrait de code).

## Proposer une Fonctionnalité

Les idées d'amélioration sont les bienvenues ! Avant de commencer à coder une fonctionnalité majeure, veuillez ouvrir une **Issue** avec le tag `enhancement` pour en discuter avec les mainteneurs. Cela évite de travailler sur une solution qui pourrait ne pas s'aligner avec la vision du projet.

## Architecture du Projet

Zephyr est un **monorepo** géré avec `pnpm`. Il est divisé en plusieurs paquets :

* **`packages/core`** : Le cœur du système. Contient le code source Rust compilé en WebAssembly (WASM) et les bindings TypeScript.
* **`packages/react-native`** : L'API React Native (composants `StyledView`, `StyledText`, le provider, etc.).
* **`packages/babel-plugin`** : Le plugin de pré-compilation statique pour analyser et extraire les classes Tailwind.
* **`examples/rnDemo`** : Une application React Native de test pour valider le comportement en conditions réelles.

## Environnement de Développement

Pour contribuer au code, vous devrez configurer votre environnement local.

### 1. Prérequis

Assurez-vous d'avoir installé les outils suivants :

* **Node.js** (v20 ou supérieur)
* **pnpm** (gestionnaire de paquets)
* **Rust** (v1.85 ou supérieur)
* **wasm-pack** (`cargo install wasm-pack`)
* **Tailwind CSS CLI** (`npm install -g tailwindcss`)

### 2. Installation initiale

Bifurquez (fork) le dépôt, clonez votre fork localement, puis installez les dépendances à la racine du monorepo :

```bash
git clone https://github.com/VOTRE_NOM_UTILISATEUR/zephyr.git
cd zephyr
pnpm install

```

### 3. Compiler le cœur Rust / WASM

C'est une étape cruciale à chaque fois que vous modifiez le code dans `packages/core`.

```bash
cd packages/core
wasm-pack build --target bundler --out-dir pkg --no-opt
pnpm run build:ts
cd ../..

```

##  Standards de Code

Nous utilisons **ESLint** et **Prettier** pour garantir une qualité de code constante. Des GitHub Actions valideront votre code lors de l'ouverture d'une Pull Request.

Avant de soumettre votre code (commit), assurez-vous de lancer ces commandes depuis la racine du projet :

* **Formater le code :**
```bash
pnpm run format

```


* **Vérifier le linting :**
```bash
pnpm run lint

```



*Note concernant Rust :* Si vous touchez au code Rust, veillez à lancer `cargo fmt` et `cargo clippy` dans le dossier `packages/core`.

## Soumettre une Pull Request (PR)

1. **Créer une branche :** Créez toujours une branche à partir de `main` avec un nom descriptif (ex: `fix/babel-plugin-crash` ou `feat/support-new-variants`).
```bash
git checkout -b type/description-courte

```


2. **Développer :** Faites vos modifications, compilez et testez-les localement (notamment via `examples/rnDemo`).
3. **Commit :** Rédigez des messages de commit clairs et concis. Nous recommandons la convention des *Conventional Commits* (`feat:`, `fix:`, `docs:`, etc.).
4. **Push & PR :** Poussez votre branche sur votre fork et ouvrez une Pull Request vers la branche `main` du dépôt original.
5. **Review :** Un mainteneur examinera votre code. Soyez ouvert aux retours et prêt à faire quelques ajustements si nécessaire.

Merci encore pour votre contribution à Zephyr !
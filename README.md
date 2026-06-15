<p align="center">
  <img src="./public/ico/zephyr.png" alt="Zephyr Vault Banner" width="100" height="100">
</p>

<h1 align="center"> Zephyr Vault</h1>

![Rust Version](https://img.shields.io/badge/Rust-2024-orange?style=for-the-badge&logo=rust)
![Security](https://img.shields.io/badge/Crypto-AES--256--GCM-blue?style=for-the-badge&logo=pre-commit)
![KDF](https://img.shields.io/badge/KDF-Argon2-lightgrey?style=for-the-badge)
![CI/CD](https://img.shields.io/github/actions/workflow/status/your-username/zephyr_vault/ci.yml?branch=main&style=for-the-badge&label=CI%2FCD&logo=github-actions)

**Zephyr Vault** est un gestionnaire de mots de passe local, ultra-rapide et sécurisé en ligne de commande (CLI). Vos informations d'identification sont stockées localement dans un fichier hautement chiffré, combinant des standards cryptographiques éprouvés et une interface utilisateur en couleur avec un rendu ASCII art soigné.

---

## ✨ Fonctionnalités

* 🔒 **Sécurité Maximale** : Chiffrement de bout en bout basé sur des algorithmes de niveau militaire.
* 🎲 **Générateur Intégré** : Création instantanée de mots de passe robustes hautement configurables (longueur, symboles).
* 📋 **Presse-papiers Sécurisé** : Copie directe de vos mots de passe via l'index ou une recherche textuelle sans les afficher à l'écran.
* 🔍 **Recherche Avancée** : Filtrage intelligent de vos comptes par service ou nom d'utilisateur.
* ⚡ **Affichage Adaptatif** : Tableaux dynamiques qui s'ajustent automatiquement à la taille de vos données textuelles.
* 📤 **Import / Export** : Sauvegarde et restauration faciles au format CSV.
* 🛠️ **Prêt pour le DevOps** : Pipeline CI/CD complet intégrant le formatage, le linting strict, ainsi qu'un audit de sécurité automatisé des dépendances.

---

## 🛠️ Architecture Technique & Sécurité

L'application repose sur une infrastructure cryptographique moderne et robuste :

| Composant | Technologie | Description |
| :--- | :--- | :--- |
| **Chiffrement Symétrique** | `AES-256-GCM` | Assure la confidentialité et l'intégrité de la base de données (`vault.enc`). |
| **Dérivation de Clé (KDF)** | `Argon2id` | Hachage sécurisé du mot de passe maître avec un sel unique de 16 octets pour résister aux attaques par force brute. |
| **CSPRNG** | `rand::rngs::OsRng` | Génération de nonces (IV) de 12 octets et de sels hautement imprévisibles via le générateur du système d'exploitation. |
| **Persistance** | `Serde` & `JSON` | Sérialisation structurée et robuste des données avant l'encodage final en `Base64`. |

---

## 🚀 Installation

### Prérequis
Assurez-vous d'avoir installé la toolchain Rust stable (`cargo`).

### Installation rapide
Exécutez le script d'installation fourni pour compiler le binaire en mode release et l'ajouter à vos exécutables globaux :

```bash
chmod +x install.sh
./install.sh

```

---

## 💡 Productivité : Raccourcis & Aliases

Pour une utilisation fluide au quotidien, ajoutez ces alias optimisés à votre fichier de configuration de shell (ex: `~/.bashrc` ou `~/.zshrc`) :

```bash
# Zephyr Vault Aliases
alias zv="zephyr_vault"
alias zadd="zephyr_vault add"
alias zlist="zephyr_vault list"
alias zsearch="zephyr_vault search"

```

Après avoir rechargé votre terminal (`source ~/.bashrc`), vous pourrez simplement utiliser `zlist` ou `zv --quiet list`.

---

## 💻 Guide d'utilisation

### Gestion des comptes

```bash
# Ajouter un compte avec génération automatique d'un mot de passe fort (16 caractères)
zephyr_vault add github mon.email@gmail.com --generate

# Ajouter un compte avec un mot de passe généré sur mesure (20 caractères avec symboles)
zephyr_vault add gitlab mon.email@gmail.com --generate --length 20 --symbols

# Ajouter un compte en saisissant manuellement le mot de passe (saisie masquée)
zephyr_vault add google mon.email@gmail.com

```

### Consultation & Recherche

```bash
# Lister tous les comptes (les mots de passe restent masqués par défaut)
zephyr_vault list

# Lister en affichant explicitement les mots de passe
zephyr_vault list --show

# Rechercher un compte spécifique
zephyr_vault search github

# Rechercher et afficher le mot de passe associé
zephyr_vault search github --show

```

### Presse-papiers & Utilitaires

```bash
# Copier un mot de passe directement dans le presse-papiers par son index
zephyr_vault copy 0

# Copier un mot de passe par une recherche textuelle directe
zephyr_vault copy github

# Générer un mot de passe fort à la volée (sans l'enregistrer dans le coffre)
zephyr_vault generate --length 24 --symbols

```

### Maintenance & Sauvegarde

```bash
# Exporter le coffre-fort en CSV déchiffré (à utiliser avec prudence !)
zephyr_vault export backup.csv

# Importer des comptes à partir d'un fichier CSV
zephyr_vault import backup.csv

# Supprimer définitivement l'intégralité du coffre-fort
zephyr_vault wipe --force

```

---

## 🤖 Pipeline CI/CD

Le projet inclut une configuration GitHub Actions (`ci.yml`) stricte exécutée à chaque push ou pull request sur les branches `main` et `master`. Elle valide les étapes suivantes :

1. **Formatage (`rustfmt`)** : Garantit la conformité du style de code selon les standards officiels de Rust.
2. **Linting (`clippy`)** : Analyse statique poussée empêchant les anti-patterns et les warnings (`-D warnings`).
3. **Audit de Sécurité (`cargo-audit`)** : Analyse l'arbre des dépendances pour détecter d'éventuelles vulnérabilités connues (CVE).
4. **Build & Test** : Compilation optimisée en mode release suivie du lancement des tests unitaires et d'un test de fumée (*smoke test*) sur l'exécutable généré.

```

```
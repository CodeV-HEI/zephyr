# Générer un mot de passe fort

zephyr_vault generate --length 20 --symbols

# Ajouter un compte avec mot de passe généré

zephyr_vault add github mon.email@gmail.com --generate --length 16

# Lister en masquant les mots de passe

zephyr_vault list

# Rechercher et afficher le mot de passe

zephyr_vault search github --show

# Exporter vers CSV (non chiffré, soyez prudent)

zephyr_vault export backup.csv

# Importer depuis CSV

zephyr_vault import backup.csv


# Afficher la bannière sans rien faire

zephyr_vault banner

# Ajouter un compte avec génération (couleurs)

zephyr_vault add github mon.email@gmail.com --generate --length 20 --symbols

# Lister avec couleurs et colonnes adaptées

zephyr_vault list

# Copier le mot de passe dans le presse-papiers (par index)

zephyr_vault copy 0

# Copier par recherche

zephyr_vault copy github

# Supprimer tout le coffre (danger)

zephyr_vault wipe --force

# Mode silencieux (pas de bannière)

zephyr_vault --quiet list
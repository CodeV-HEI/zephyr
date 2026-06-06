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
# Vérification du TUI

Le test déterministe `cargo test --locked --test tui_render` dessine l'accueil, la liste et l'éditeur dans le backend mémoire de Ratatui. Il vérifie deux tailles utilisables (`100×32` et `60×20`) et une taille trop petite (`30×8`), redessine chaque écran deux fois et compare les résultats. Il vérifie aussi la couleur d'accent et que la compilation laisse l'interface réactive jusqu'au résultat.

Sur Linux et macOS, `python3 tests/tui_pty.py target/release/forge` ouvre un vrai pseudo-terminal. Il teste l'accueil, l'aide, la navigation jusqu'à l'éditeur et la restauration du terminal à la sortie sous `TERM=xterm`, `xterm-256color` et `screen-256color`. La CI l'exécute après le build release sur les runners Unix. Pour l'exécuter localement :

```sh
cargo build --release --locked
python3 tests/tui_pty.py target/release/forge
```

Ces profils contrôlent les séquences et le clavier via un PTY. Ils ne simulent pas les polices, thèmes ou touches physiques des émulateurs. Le runner Windows exécute le test mémoire ; une vérification Windows Terminal réelle reste nécessaire.

La vérification manuelle complète ce test sur chaque terminal visé. Depuis la racine du dépôt, lancer `cargo run -- start` et suivre la même séquence dans GNOME Terminal, macOS Terminal et Windows Terminal :

1. À `100×32`, sélectionner Java puis Rust avec les flèches, ouvrir un exercice et vérifier que l'objectif, le code, les diagnostics et le résultat sont visibles.
2. À environ `60×20`, vérifier que les panneaux de diagnostic et de résultat s'empilent et que F1, F2, Ctrl+S, Ctrl+R et Esc fonctionnent.
3. À moins de `40×12`, vérifier que le message demandant d'agrandir le terminal remplace l'interface comprimée.
4. Saisir du texte Unicode, déclencher une erreur de compilation, puis un programme à sortie incorrecte. Vérifier le curseur, les couleurs, la lisibilité des messages et le retour à la liste.

Noter pour chaque terminal sa version, sa taille, les raccourcis qui échouent et une capture de l'écran concerné. Les tests automatiques garantissent un rendu mémoire reproductible et un parcours PTY reproductible sur Unix ; la lisibilité visuelle et les touches physiques restent des contrôles humains sur chaque émulateur cible. Aucun émulateur macOS ou Windows n'est disponible dans l'environnement de développement Linux.

# VHS Recording Script for "airis init" Demo
# https://github.com/charmbracelet/vhs
#
# Install: brew install vhs
# Record:  vhs 01-init-demo.tape
# Output:  airis-init-demo.gif

# Configuration
Output airis-init-demo.gif
Set FontSize 18
Set Width 900
Set Height 500
Set Theme "Catppuccin Mocha"
Set Padding 20
Set TypingSpeed 50ms

# Start in temp directory
Hide
Type "cd /tmp && rm -rf airis-demo && mkdir airis-demo && cd airis-demo"
Enter
Sleep 500ms
Show

# Show empty directory
Type "ls -la"
Enter
Sleep 1s

# Show the magic command
Type "# One command to generate your entire monorepo config"
Enter
Sleep 1s

# Run airis init
Type "airis init"
Enter
Sleep 3s

# Show generated files
Type "ls -la"
Enter
Sleep 1s

# Show tree structure
Type "tree -a -I 'node_modules|.git'"
Enter
Sleep 2s

# Optional: Show manifest.toml is the source
Type "# manifest.toml is the single source of truth"
Enter
Sleep 500ms
Type "head -20 manifest.toml"
Enter
Sleep 2s

# End
Sleep 1s

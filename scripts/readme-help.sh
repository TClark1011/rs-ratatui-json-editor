# we embed the app's `--help` output into the README as
# documentation

cargo run -- --help > help_output.tmp

# echo "$HELP_OUTPUT"

# Define start and end markers in README
START_MARKER="<!-- HELP_OUTPUT_START -->"
END_MARKER="<!-- HELP_OUTPUT_END -->"

# Update the README file in place
awk -v start_marker="$START_MARKER" -v end_marker="$END_MARKER" '
    BEGIN { in_section=0 }
    $0 ~ start_marker { print; print ""; print "```"; while ((getline < "help_output.tmp") > 0) print; print "```"; print ""; in_section=1; next }
    $0 ~ end_marker { in_section=0 }
    !in_section { print }
' README.md > README.tmp && mv README.tmp README.md

# Remove the temporary file
rm help_output.tmp

# Add README changes to the commit
git add README.md
# tell git to track changes to the test files
git update-index --no-assume-unchanged $(find . -type f -path '*/test-files/*.json')
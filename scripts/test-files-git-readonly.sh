# tell git to ignore changes to the test files
git update-index --assume-unchanged $(find . -type f -path '*/test-files/*.json')
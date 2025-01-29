mkdir -p bin
gcc -Wall -g -O0 src/main.c -o bin/main -Isrc/ -L../../target/release/ -lkagippcore

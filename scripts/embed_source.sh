#!/bin/bash
# Embed entire Fast Forth source code into the binary
# This makes the binary completely self-contained and auditable

set -e

echo "Embedding Fast Forth source code into binary..."

# Create embedded source directory
mkdir -p build/embedded

# Archive entire source (excluding build artifacts and git)
tar czf build/embedded/source.tar.gz \
    --exclude='target' \
    --exclude='.git' \
    --exclude='build' \
    --exclude='*.o' \
    --exclude='*.a' \
    --exclude='*.so' \
    --exclude='*.dylib' \
    .

SOURCE_SIZE=$(wc -c < build/embedded/source.tar.gz | tr -d ' ')
echo "Source archive size: ${SOURCE_SIZE} bytes (~$((SOURCE_SIZE / 1024)) KB)"

# Convert to C array
echo "Converting to C array..."
xxd -i build/embedded/source.tar.gz > build/embedded/embedded_source.c

# Add extraction function
cat >> build/embedded/embedded_source.c << 'EOF'

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int extract_embedded_source(const char *output_dir) {
    FILE *f = fopen("embedded_source.tar.gz", "wb");
    if (!f) {
        fprintf(stderr, "Error: Cannot create embedded_source.tar.gz\n");
        return -1;
    }

    fwrite(build_embedded_source_tar_gz, 1,
           build_embedded_source_tar_gz_len, f);
    fclose(f);

    printf("Embedded source extracted to: embedded_source.tar.gz\n");
    printf("Size: %u bytes\n", build_embedded_source_tar_gz_len);
    printf("\nTo rebuild from source:\n");
    printf("  tar xzf embedded_source.tar.gz\n");
    printf("  cd fast-forth\n");
    printf("  make -C minimal_forth    # Minimal compiler (30s, no Rust)\n");
    printf("  # OR\n");
    printf("  ./install-rust.sh        # Install Rust (5-25 min)\n");
    printf("  cargo build --release    # Full optimizations (2-5 min)\n");
    printf("\n");

    return 0;
}
EOF

echo "Source embedding complete!"
echo "Embedded source can be extracted with: fastforth --extract-source"

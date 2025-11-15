\ Fast Forth Build Orchestrator - Pure Forth
\ Replaces shell scripts with Forth implementation

\ Rust detection
: rust-installed? ( -- flag )
  s" which cargo" system 0= ;

: rust-version ( -- )
  s" rustc --version" system-output type cr ;

\ Build modes
: compile-with-tinycc ( -- )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Building with TinyCC Fallback" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." Performance: 60-75% of C (fast compilation)" cr
  ." Compile time: 5-10ms" cr
  cr

  ." Compiling C runtime with TinyCC..." cr
  s" tinycc/tcc -O2 runtime/*.c -o fastforth-tinycc" system
  if
    ." ✗ Build failed" cr
  else
    ." ✓ Build complete!" cr
    cr
    ." Binary: ./fastforth-tinycc" cr
    ." Size: " s" du -h fastforth-tinycc | cut -f1" system-output type cr
    ." Performance: 60-75% of C" cr
    cr
  then ;

: download-rustup ( -- success? )
  cr
  ." Downloading Rust installer..." cr
  s" curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o /tmp/rustup.sh" system 0=
  dup if
    ." ✓ Downloaded rustup" cr
  else
    ." ✗ Download failed" cr
  then ;

: install-rust ( -- success? )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Installing Rust Toolchain" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." This will enable full optimizations (85-110% of C)" cr
  ." Download size: ~1.5 GB" cr
  ." Install time: 5-25 minutes" cr
  cr

  ." Install Rust now? (y/n): "
  key
  dup emit cr

  [char] y = if
    download-rustup
    if
      s" sh /tmp/rustup.sh -y" system 0=
      dup if
        s" source $HOME/.cargo/env" system drop
        ." ✓ Rust installed successfully!" cr
        cr
        rust-version
      else
        ." ✗ Rust installation failed" cr
      then
    else
      false
    then
  else
    ." Rust installation cancelled" cr
    false
  then ;

: compile-with-cargo ( -- )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Building with Rust+LLVM" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." Performance: 85-110% of C (full optimizations)" cr
  ." Compile time: 2-5 minutes" cr
  cr

  ." Building..." cr
  s" cargo build --release" system
  if
    ." ✗ Build failed" cr
  else
    ." ✓ Build complete!" cr
    cr
    ." Binary: target/release/fastforth" cr
    ." Size: " s" du -h target/release/fastforth | cut -f1" system-output type cr
    ." Performance: 85-110% of C" cr
    cr
    ." Install globally:" cr
    ."   cargo install --path ." cr
    cr
  then ;

: compile-optimized ( -- )
  rust-installed?
  if
    ." Rust found: " rust-version
    compile-with-cargo
  else
    ." Rust not found" cr
    install-rust
    if
      compile-with-cargo
    else
      cr
      ." Using fallback compiler instead" cr
      compile-with-tinycc
    then
  then ;

\ Main compile command
: compile ( optimized? -- )
  if
    compile-optimized
  else
    compile-with-tinycc
  then ;

\ CLI integration
: --compile
  false compile ;  \ Default: fallback mode

: --compile-optimized
  true compile ;   \ Optimized: download Rust if needed

\ Export
: --compile --compile ;
: --optimized --compile-optimized ;

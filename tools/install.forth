\ Fast Forth Installer - Pure Forth Implementation
\ No shell scripts!

\ Platform detection
variable platform-id
variable arch-name
variable os-name

: detect-os ( -- os-code )
  \ 0=macOS, 1=Linux, 2=Windows
  s" uname -s" system-output
  s" Darwin" compare 0= if 0 exit then
  s" Linux" compare 0= if 1 exit then
  2 ;  \ Windows

: detect-arch ( -- arch-code )
  \ 0=ARM64, 1=x86_64
  s" uname -m" system-output
  dup s" arm64" compare 0= if drop 0 exit then
  s" aarch64" compare 0= if 0 exit then
  1 ;  \ x86_64

: platform-string ( os arch -- addr len )
  swap
  case
    0 of  \ macOS
      0= if s" aarch64-apple-darwin" else s" x86_64-apple-darwin" then
    endof
    1 of  \ Linux
      0= if s" aarch64-unknown-linux-gnu" else s" x86_64-unknown-linux-gnu" then
    endof
    2 of  \ Windows
      drop s" x86_64-pc-windows-msvc"
    endof
  endcase ;

\ GitHub API interaction
: github-release-url ( -- addr len )
  s" https://github.com/quivent/fast-forth/releases/latest/download/fastforth-"
  detect-os detect-arch platform-string
  concat ;

: download-file ( url-addr url-len dest-addr dest-len -- success? )
  \ Use built-in HTTP client or system curl
  2>r 2dup
  s" curl -L -o " 2r> concat concat
  system 0= ;

: verify-checksum ( file -- success? )
  \ TODO: Implement SHA256 verification
  drop true ;

\ Installation
: install-binary ( -- )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Fast Forth Installer" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." Platform: " detect-os detect-arch platform-string type cr
  cr
  ." Downloading Fast Forth binary..." cr

  github-release-url s" fastforth" download-file
  if
    ." ✓ Download complete" cr
    cr
    ." Making executable..." cr
    s" chmod +x fastforth" system drop
    ." ✓ Installation complete!" cr
    cr
    ." Binary: ./fastforth" cr
    ." Size: " s" du -h fastforth | cut -f1" system-output type cr
    cr
    ." Test it:" cr
    ."   ./fastforth --version" cr
    ."   ./fastforth repl" cr
    cr
  else
    ." ✗ Download failed" cr
    ." Please check your internet connection" cr
  then ;

\ Main entry point
: install
  install-binary ;

\ Auto-run if executed directly
install bye

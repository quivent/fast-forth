\ Fast Forth Cross-Platform Installer
\ Works on: macOS, Linux (Ubuntu/Debian/Fedora/Arch), Windows

needs platform.forth  \ Load platform detection

\ GitHub API
: github-base-url ( -- addr len )
  s" https://github.com/quivent/fast-forth/releases/latest/download/" ;

: binary-filename ( -- addr len )
  s" fastforth-"
  current-platform @ current-arch @ platform-triple concat
  current-platform @ PLATFORM-WINDOWS = if
    s" .exe" concat
  then ;

: download-url ( -- addr len )
  github-base-url binary-filename concat ;

\ Download strategies per platform
: download-with-curl ( url dest -- success? )
  2swap 2over
  s" curl -L -o " 2swap concat
  s"  " concat 2swap concat
  system 0= ;

: download-with-wget ( url dest -- success? )
  2swap 2over
  s" wget -O " 2swap concat
  s"  " concat 2swap concat
  system 0= ;

: download-with-powershell ( url dest -- success? )
  \ Windows PowerShell download
  2swap 2over
  s" powershell -Command \"Invoke-WebRequest -Uri '" 2swap concat
  s" ' -OutFile '" concat 2swap concat
  s" '\"" concat
  system 0= ;

: download-file ( url dest -- success? )
  is-windows? if
    download-with-powershell
  else
    \ Try curl first, fallback to wget
    2dup download-with-curl
    if 2drop true
    else download-with-wget
    then
  then ;

\ Permission setting
: make-executable ( file -- )
  is-unix? if
    s" chmod +x " 2swap concat system drop
  then ;  \ Windows doesn't need chmod

\ Installation
: check-dependencies ( -- success? )
  is-unix? if
    \ Check for curl or wget
    s" which curl" system 0=
    s" which wget" system 0= or
  else
    \ Windows always has PowerShell
    true
  then ;

: install-dependencies ( -- )
  cr
  ." Installing dependencies..." cr
  install-dependencies-cmd system drop
  ." ✓ Dependencies installed" cr ;

: verify-binary ( file -- success? )
  \ Check if file exists and is executable
  2dup s" test -f " 2swap concat system 0= if
    is-unix? if
      s" test -x " 2swap concat system 0=
    else
      2drop true  \ Windows files don't have executable bit
    then
  else
    2drop false
  then ;

: install-binary ( -- success? )
  cr
  ." ═══════════════════════════════════════════════════════════" cr
  ."   Fast Forth Cross-Platform Installer" cr
  ." ═══════════════════════════════════════════════════════════" cr
  cr

  ." Platform: " current-platform @ platform-name type
  ."  (" current-arch @ arch-name type ." )" cr
  ." Binary: " binary-filename type cr
  cr

  \ Check dependencies
  check-dependencies 0= if
    ." Dependencies missing. Install? (y/n): "
    key [char] y = if
      install-dependencies
    else
      ." Installation cancelled." cr
      false exit
    then
  then

  ." Downloading Fast Forth binary..." cr
  download-url s" fastforth" executable-name download-file

  if
    ." ✓ Download complete" cr
    cr

    \ Make executable (Unix only)
    s" fastforth" executable-name make-executable

    \ Verify
    s" fastforth" executable-name verify-binary
    if
      ." ✓ Installation successful!" cr
      cr
      ." Binary: ./fastforth" current-platform @ binary-suffix concat type cr
      ." Size: "
      is-windows? if
        s" dir /n fastforth.exe" system-output
      else
        s" du -h fastforth | cut -f1" system-output
      then
      type cr
      cr
      ." Test it:" cr
      ."   ./fastforth" current-platform @ binary-suffix concat type ."  --version" cr
      ."   ./fastforth" current-platform @ binary-suffix concat type ."  repl" cr
      cr
      true
    else
      ." ✗ Verification failed" cr
      false
    then
  else
    ." ✗ Download failed" cr
    cr
    ." Please check:" cr
    ."   1. Internet connection" cr
    ."   2. GitHub is accessible" cr
    ."   3. Release exists: " download-url type cr
    cr
    false
  then ;

\ Platform-specific instructions
: platform-instructions ( -- )
  cr
  current-platform @ case
    PLATFORM-LINUX of
      ." Linux Installation:" cr
      ." ─────────────────────" cr
      ." Run this installer OR download manually:" cr
      cr
      ." Ubuntu/Debian:" cr
      ."   sudo apt-get update" cr
      ."   sudo apt-get install -y curl" cr
      ."   curl -L https://raw.githubusercontent.com/quivent/fast-forth/main/tools/install.forth | forth" cr
      cr
      ." Fedora/CentOS:" cr
      ."   sudo dnf install -y curl" cr
      ."   curl -L https://raw.githubusercontent.com/quivent/fast-forth/main/tools/install.forth | forth" cr
      cr
      ." Arch Linux:" cr
      ."   sudo pacman -Sy curl" cr
      ."   curl -L https://raw.githubusercontent.com/quivent/fast-forth/main/tools/install.forth | forth" cr
    endof

    PLATFORM-MACOS of
      ." macOS Installation:" cr
      ." ───────────────────" cr
      ." Run this installer OR use Homebrew:" cr
      cr
      ."   brew tap quivent/fast-forth" cr
      ."   brew install fast-forth" cr
    endof

    PLATFORM-WINDOWS of
      ." Windows Installation:" cr
      ." ────────────────────" cr
      ." Run this installer OR download manually:" cr
      cr
      ." PowerShell:" cr
      ."   Invoke-WebRequest -Uri https://github.com/quivent/fast-forth/releases/latest/download/fastforth-x86_64-pc-windows-msvc.exe -OutFile fastforth.exe" cr
      cr
      ." Or use winget (if available):" cr
      ."   winget install QuiVent.FastForth" cr
    endof

    drop
    ." Generic Installation:" cr
    ." ────────────────────" cr
    ." Download from:" cr
    ."   https://github.com/quivent/fast-forth/releases" cr
  endcase
  cr ;

\ Main entry point
: install
  init-platform
  install-binary
  if
    ." ═══════════════════════════════════════════════════════════" cr
    ."   Installation Complete!" cr
    ." ═══════════════════════════════════════════════════════════" cr
  else
    ." ═══════════════════════════════════════════════════════════" cr
    ."   Installation Failed" cr
    ." ═══════════════════════════════════════════════════════════" cr
    cr
    platform-instructions
  then ;

\ Export commands
: --install install ;
: --help platform-instructions ;

\ Auto-run if executed directly
install bye

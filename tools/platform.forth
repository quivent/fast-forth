\ Cross-Platform Detection for Fast Forth
\ Works on: macOS, Linux (Ubuntu/Debian/Fedora/Arch), Windows

\ Platform codes
0 constant PLATFORM-MACOS
1 constant PLATFORM-LINUX
2 constant PLATFORM-WINDOWS
3 constant PLATFORM-BSD
4 constant PLATFORM-UNKNOWN

\ Architecture codes
0 constant ARCH-ARM64
1 constant ARCH-X86-64
2 constant ARCH-X86
3 constant ARCH-UNKNOWN

\ Global platform state
variable current-platform
variable current-arch
variable platform-detected?

\ System command execution
: system-output ( cmd-addr cmd-len -- output-addr output-len )
  \ Execute system command and capture output
  \ Implementation depends on Forth system
  2dup type cr
  s" " ;  \ Placeholder - returns empty string

\ OS Detection
: detect-os-name ( -- os-code )
  \ Try uname first (Unix-like systems)
  s" uname -s 2>/dev/null" system-output

  \ Check for Darwin (macOS)
  2dup s" Darwin" compare 0= if 2drop PLATFORM-MACOS exit then

  \ Check for Linux
  2dup s" Linux" compare 0= if 2drop PLATFORM-LINUX exit then

  \ Check for FreeBSD/OpenBSD/NetBSD
  2dup s" BSD" search nip nip if 2drop PLATFORM-BSD exit then

  2drop

  \ Try Windows detection via ver command
  s" ver 2>nul" system-output
  2dup s" Windows" search nip nip if
    2drop PLATFORM-WINDOWS exit
  then
  2drop

  \ Try checking for Windows via environment variables
  s" echo %OS%" system-output
  2dup s" Windows" compare 0= if
    2drop PLATFORM-WINDOWS exit
  then
  2drop

  PLATFORM-UNKNOWN ;

\ Architecture Detection
: detect-architecture ( -- arch-code )
  \ Try uname -m (Unix-like)
  s" uname -m 2>/dev/null" system-output

  \ ARM64 variants
  2dup s" arm64" compare 0= if 2drop ARCH-ARM64 exit then
  2dup s" aarch64" compare 0= if 2drop ARCH-ARM64 exit then
  2dup s" armv8" compare 0= if 2drop ARCH-ARM64 exit then

  \ x86-64 variants
  2dup s" x86_64" compare 0= if 2drop ARCH-X86-64 exit then
  2dup s" amd64" compare 0= if 2drop ARCH-X86-64 exit then

  \ x86 32-bit
  2dup s" i386" compare 0= if 2drop ARCH-X86 exit then
  2dup s" i686" compare 0= if 2drop ARCH-X86 exit then

  2drop

  \ Windows: use PROCESSOR_ARCHITECTURE
  s" echo %PROCESSOR_ARCHITECTURE%" system-output
  2dup s" AMD64" compare 0= if 2drop ARCH-X86-64 exit then
  2dup s" x86" compare 0= if 2drop ARCH-X86 exit then
  2drop

  ARCH-UNKNOWN ;

\ Platform string generation
: platform-name ( os-code -- addr len )
  case
    PLATFORM-MACOS of s" macOS" endof
    PLATFORM-LINUX of s" Linux" endof
    PLATFORM-WINDOWS of s" Windows" endof
    PLATFORM-BSD of s" BSD" endof
    s" Unknown"
  endcase ;

: arch-name ( arch-code -- addr len )
  case
    ARCH-ARM64 of s" ARM64" endof
    ARCH-X86-64 of s" x86-64" endof
    ARCH-X86 of s" x86" endof
    s" Unknown"
  endcase ;

\ Binary filename generation
: binary-suffix ( os-code -- addr len )
  case
    PLATFORM-WINDOWS of s" .exe" endof
    s" "  \ No suffix for Unix-like
  endcase ;

: platform-triple ( os-code arch-code -- addr len )
  \ Generate Rust target triple
  swap
  case
    PLATFORM-MACOS of
      case
        ARCH-ARM64 of s" aarch64-apple-darwin" endof
        ARCH-X86-64 of s" x86_64-apple-darwin" endof
        s" unknown-apple-darwin"
      endcase
    endof

    PLATFORM-LINUX of
      case
        ARCH-ARM64 of s" aarch64-unknown-linux-gnu" endof
        ARCH-X86-64 of s" x86_64-unknown-linux-gnu" endof
        ARCH-X86 of s" i686-unknown-linux-gnu" endof
        s" unknown-linux-gnu"
      endcase
    endof

    PLATFORM-WINDOWS of
      case
        ARCH-X86-64 of s" x86_64-pc-windows-msvc" endof
        ARCH-X86 of s" i686-pc-windows-msvc" endof
        s" unknown-pc-windows-msvc"
      endcase
    endof

    PLATFORM-BSD of
      case
        ARCH-X86-64 of s" x86_64-unknown-freebsd" endof
        s" unknown-freebsd"
      endcase
    endof

    drop s" unknown-unknown-unknown"
  endcase ;

\ Package manager detection (for Linux)
: detect-package-manager ( -- pm-code )
  \ 0=apt, 1=dnf/yum, 2=pacman, 3=zypper, 4=unknown
  s" which apt-get 2>/dev/null" system-output
  nip if 0 exit then

  s" which dnf 2>/dev/null" system-output
  nip if 1 exit then

  s" which yum 2>/dev/null" system-output
  nip if 1 exit then

  s" which pacman 2>/dev/null" system-output
  nip if 2 exit then

  s" which zypper 2>/dev/null" system-output
  nip if 3 exit then

  4 ;

\ Dependency installation commands
: install-dependencies-cmd ( -- addr len )
  current-platform @ PLATFORM-LINUX = if
    detect-package-manager
    case
      0 of s" sudo apt-get update && sudo apt-get install -y build-essential curl" endof
      1 of s" sudo dnf groupinstall -y 'Development Tools' && sudo dnf install -y curl" endof
      2 of s" sudo pacman -Sy --noconfirm base-devel curl" endof
      3 of s" sudo zypper install -y -t pattern devel_basis && sudo zypper install -y curl" endof
      s" echo 'Unknown package manager - install build-essential and curl manually'"
    endcase
  else current-platform @ PLATFORM-MACOS = if
    s" xcode-select --install"
  else current-platform @ PLATFORM-WINDOWS = if
    s" echo 'Install Visual Studio Build Tools from https://visualstudio.microsoft.com/downloads/'"
  else
    s" echo 'Unknown platform - cannot install dependencies'"
  then then then ;

\ TinyCC availability check
: tinycc-available? ( -- flag )
  current-platform @ PLATFORM-WINDOWS = if
    \ TinyCC works on Windows
    true
  else current-platform @ PLATFORM-LINUX = if
    \ TinyCC works on Linux
    true
  else current-platform @ PLATFORM-MACOS = if
    \ TinyCC has limited macOS support
    \ Use only if explicitly available
    s" which tcc 2>/dev/null" system-output nip 0>
  else
    false
  then then then ;

\ Platform initialization
: init-platform ( -- )
  platform-detected? @ if exit then  \ Already detected

  detect-os-name current-platform !
  detect-architecture current-arch !
  true platform-detected? !

  \ Display detected platform
  cr
  ." Platform: " current-platform @ platform-name type
  ."  (" current-arch @ arch-name type ." )" cr
  ." Target: " current-platform @ current-arch @ platform-triple type cr
  cr ;

\ Utility functions
: is-unix? ( -- flag )
  current-platform @
  dup PLATFORM-MACOS =
  swap dup PLATFORM-LINUX =
  swap PLATFORM-BSD =
  or or ;

: is-windows? ( -- flag )
  current-platform @ PLATFORM-WINDOWS = ;

: path-separator ( -- char )
  is-windows? if [char] \ else [char] / then ;

: executable-name ( base-addr base-len -- full-addr full-len )
  \ Add .exe on Windows
  is-windows? if
    s" .exe" concat
  then ;

\ Export
: --platform init-platform ;

\ Auto-initialize on load
init-platform

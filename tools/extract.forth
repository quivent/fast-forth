\ Fast Forth Source Extractor - Pure Forth
\ Extracts and decompresses embedded source automatically

\ Embedded source location (populated at build time)
variable embedded-source-addr
variable embedded-source-len

\ Decompression
: decompress-gzip ( addr len -- decompressed-addr decompressed-len )
  \ Use zlib FFI or system gunzip
  \ For now, write to temp file and decompress
  s" /tmp/embedded.tar.gz" write-file
  s" gunzip -c /tmp/embedded.tar.gz > /tmp/embedded.tar" system drop
  s" /tmp/embedded.tar" read-file ;

: extract-tar ( addr len dest -- )
  \ Extract tar archive to destination directory
  >r
  s" /tmp/to-extract.tar" write-file
  s" tar xf /tmp/to-extract.tar -C " r> concat system drop ;

\ Automatic extraction
: extract-source ( -- )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Extracting Fast Forth Source Code" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." Binary contains complete source code (embedded)" cr
  ." Extracting to: ./fast-forth/" cr
  cr

  \ Get embedded source from binary
  embedded-source-addr @ embedded-source-len @

  \ Decompress gzip
  decompress-gzip

  \ Extract tar to current directory
  s" ." extract-tar

  cr
  ." ✓ Source extracted successfully!" cr
  cr
  ." Directory: ./fast-forth/" cr
  ." Size: " s" du -sh fast-forth | cut -f1" system-output type cr
  cr
  ." Build with fallback compiler:" cr
  ."   cd fast-forth" cr
  ."   ./fastforth --compile" cr
  cr
  ." Build with full optimizations:" cr
  ."   cd fast-forth" cr
  ."   ./fastforth --compile --optimized" cr
  cr ;

\ View source inline
: list-source-files ( -- )
  \ List files in embedded tar
  embedded-source-addr @ embedded-source-len @
  decompress-gzip
  s" tar tf -" system ;

: view-source-file ( filename-addr filename-len -- )
  \ Extract and display single file from embedded tar
  embedded-source-addr @ embedded-source-len @
  decompress-gzip
  2swap s" tar xOf - " swap concat concat
  system ;

: source ( -- )
  cr
  ." ═══════════════════════════════════════════════════════" cr
  ."   Fast Forth Embedded Source" cr
  ." ═══════════════════════════════════════════════════════" cr
  cr

  ." Available files:" cr
  list-source-files
  cr

  ." Enter filename to view (or 'all' to extract): "
  pad 256 accept
  pad swap

  dup s" all" compare 0= if
    2drop extract-source
  else
    view-source-file
  then ;

\ Export commands
: --extract extract-source ;
: --source source ;

\ Test file creation
: test-create
  s" /tmp/fastforth-test.txt" w/o create-file
  if
    ." Error creating file" cr
  else
    ." File created successfully!" cr
    close-file drop
  then
;

test-create

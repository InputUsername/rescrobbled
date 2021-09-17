#!/usr/bin/env python

import sys

# Filter scripts receive the track artist, title and album
# on separate lines of their standard input...

artist, title, album = (l.rstrip() for l in sys.stdin.readlines())

# ...and should provide them on the corresponding lines of the
# standard output

print(artist, title, album, end='\n')

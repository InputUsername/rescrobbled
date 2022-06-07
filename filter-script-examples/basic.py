#!/usr/bin/env python

import sys

# Filter scripts receive the track artist, title, album and comma-separated list of genre(s)
# on separate lines of their standard input...

artist, title, album, genres = (l.rstrip() for l in sys.stdin.readlines())
genres = genres.split(',')

# ...and should provide artist, title and album on the corresponding lines of the
# standard output

print(artist, title, album, sep='\n')

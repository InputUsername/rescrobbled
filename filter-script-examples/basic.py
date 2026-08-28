#!/usr/bin/env python

import sys

# Filter scripts receive the track artist, title, album, comma-separated list of genre(s)
# and album artist on separate lines of their standard input...

artist, title, album, genres, album_artist = (l.rstrip() for l in sys.stdin.readlines())
genres = genres.split(',')

# ...and should provide artist, title, album and album artist on the corresponding lines
# of the standard output

print(artist, title, album, album_artist, sep='\n')

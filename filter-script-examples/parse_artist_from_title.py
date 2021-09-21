#!/usr/bin/env python

import sys

artist, title, album = (l.rstrip() for l in sys.stdin.readlines())

# Parse the artist from the track title if the artist is empty

if len(artist) == 0:
    artist, title = title.split(' - ', maxsplit=1)

print(artist, title, album, sep='\n')

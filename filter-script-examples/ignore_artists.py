#!/usr/bin/env python

import sys

artist, title, album = (l.rstrip() for l in sys.stdin.readlines())

# Ignore all tracks by specific artists

IGNORED_ARTISTS = {'Justin Bieber', 'The Beatles', 'Michael Jackson'}

if artist not in IGNORED_ARTISTS:
    print(artist, title, album, end='\n')

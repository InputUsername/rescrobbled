#!/usr/bin/env python

import sys

artist, title, album, genres = (l.rstrip() for l in sys.stdin.readlines())
genres = genres.lower().split(',')

IGNORED_GENRES = {'country', 'idm'}

# Only output if none of the genres are in the IGNORED_GENRES
if all(genre not in IGNORED_GENRES for genre in genres):
    print(artist, title, album, sep='\n')

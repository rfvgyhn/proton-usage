#!/bin/bash

tail -n +2 ../CHANGELOG.md |          # Remove header
  sed '/./,$!d' |                     # Remove whitespace lines
  sed '1d' |                          # Remove release heading
  sed '/./,$!d' |                     # Remove whitespace lines
  sed '/^##[^#]\|\[unreleased\]:/q' | # Stop at next release heading or link section
  sed '$ d'                           # Remove last line
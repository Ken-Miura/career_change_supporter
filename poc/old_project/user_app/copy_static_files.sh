#!/bin/bash

DSTDIR=../../server/service/static/user
if [ -d $DSTDIR ]; then
  rm -r $DSTDIR
fi
mkdir -p $DSTDIR
cp -r ./dist/* $DSTDIR

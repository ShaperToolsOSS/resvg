#!/usr/bin/env bash
sed -i "s/yyyymmdd/$(date '+%B %d, %Y')/g" index.html
npm run build
sed -i "s/$(date '+%B %d, %Y')/yyyymmdd/g" index.html
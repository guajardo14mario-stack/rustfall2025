#!/usr/bin/env bash
set -eu
mkdir -p data
if [ ! -f gutenberg_urls.txt ]; then
  echo "Create gutenberg_urls.txt with one direct text URL per line (>=100 lines)."
  exit 1
fi

i=0
while IFS= read -r url; do
  ((i++))
  fname="data/book_${i}.txt"
  echo "Downloading #$i -> $fname from $url"
  # Use curl or wget — fall back to curl
  if command -v curl >/dev/null 2>&1; then
    curl -L --fail --silent --show-error "$url" -o "$fname" || { echo "Failed to download $url"; rm -f "$fname"; }
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$fname" "$url" || { echo "Failed to download $url"; rm -f "$fname"; }
  else
    echo "Install curl or wget"
    exit 1
  fi
done < gutenberg_urls.txt

echo "Downloaded $i files to data/"

# conv

> Universal smart CLI file converter routing media, documents, ebooks, images, and data.
> More information: <https://github.com/ACEECA1/anyconv>.

- Convert a video to an audio file (e.g. extract MP3):
  `conv {{path/to/video.mp4}} {{path/to/audio.mp3}}`

- Convert a file using shorthand target extension:
  `conv {{path/to/song.wav}} {{mp3}}`

- Batch convert all PNG images to WebP format:
  `conv '{{*.png}}' {{webp}}`

- Convert a Markdown document to HTML or DOCX:
  `conv {{path/to/document.md}} {{html}}`

- Convert an EPUB book to MOBI or PDF:
  `conv {{path/to/book.epub}} {{mobi}}`

- Convert tabular data between CSV, TSV, and JSON in-memory:
  `conv {{path/to/data.csv}} {{json}}`

- Preview planned conversion commands without executing (dry-run):
  `conv --dry-run {{path/to/video.mp4}} {{gif}}`

- Convert files in parallel using a custom number of worker threads:
  `conv -j {{8}} '{{*.jpg}}' {{webp}}`

- List all supported formats and backend engines:
  `conv --list`

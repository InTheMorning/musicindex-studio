# Test Media Files

This folder holds the media files for the tests of `media.rs`. Each file is
below 30 KB. `ffmpeg` version n9.0.2 and LAME version 4.0 made each file.

## The Source

Make one second of silence, at 8000 Hz, in one channel:

```sh
ffmpeg -f lavfi -i anullsrc=r=8000:cl=mono -t 1 -c:a pcm_s16le source.wav
```

## The Tagged Files

Each command below sets this title, artist, album and track number:

- Title: `Test Title`
- Artist: `Test Artist`
- Album: `Test Album`
- Track number: `3`

Make the tagged WAV file:

```sh
ffmpeg -i source.wav -metadata title="Test Title" -metadata artist="Test Artist" \
  -metadata album="Test Album" -metadata track=3 -c:a pcm_s16le tagged.wav
```

Make the tagged M4A file:

```sh
ffmpeg -i source.wav -metadata title="Test Title" -metadata artist="Test Artist" \
  -metadata album="Test Album" -metadata track=3 -c:a aac tagged.m4a
```

Make the tagged FLAC file:

```sh
ffmpeg -i source.wav -metadata title="Test Title" -metadata artist="Test Artist" \
  -metadata album="Test Album" -metadata track=3 -c:a flac tagged.flac
```

Make the tagged Ogg Opus file:

```sh
ffmpeg -i source.wav -metadata title="Test Title" -metadata artist="Test Artist" \
  -metadata album="Test Album" -metadata track=3 -c:a libopus tagged.opus
```

Make the tagged MP3 file, with LAME:

```sh
lame --tt "Test Title" --ta "Test Artist" --tl "Test Album" --tn 3 source.wav tagged.mp3
```

## The Other Files

Make the MP3 file with no tags. The test uses it for the title fallback:

```sh
lame source.wav untagged.mp3
```

Make the 1x1 PNG file:

```sh
ffmpeg -f lavfi -i color=c=black:s=1x1 -frames:v 1 image.png
```

`not-media.txt` is a plain text file. Studio does not accept its content as
a media type.

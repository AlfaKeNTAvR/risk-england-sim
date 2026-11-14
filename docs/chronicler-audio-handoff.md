# Handoff: organise audio for The Chronicler

You are helping Nikita collect and organise audio files on his computer for **The Chronicler**, a web page that plays music, sound effects and narration during a house-ruled game of *Risk: The Battle for Medieval England*. Another Claude session builds the page. Nikita carries files and messages between you. You do not need to write any page code.

## Goal

Fill the folder structure below with good, free, licence-safe audio. Check every file. Produce a manifest that the page-building session can use without listening to anything.

## Folder structure (create it if missing)

```
Chronicler/
├── music/
│   ├── suspense/      placing troops, between battles: tense, expectant, no full battle yet
│   ├── battle/        dice rolling: drums, driving orchestral, high energy
│   ├── early-ages/    5th–7th century, calm: lute, harp, recorder, gentle folk
│   ├── middle-ages/   8th–11th century, steady: marches, knights, fuller sound
│   ├── late-ages/     12th–14th century, dark and heavy
│   ├── good-age/      festive events: tavern, dance, festival
│   ├── order/         Magna Carta, Age of saints, One kingdom: chant, choir, royal court
│   ├── intrigue/      Danegeld, The Anarchy: mysterious, scheming
│   ├── dark-age/      plague, famine, hard winter: sad, slow, lament
│   ├── war-age/       Viking raids, Barons' War: epic, martial (but not drums-only)
│   └── finale/        game over, final count: triumphant
└── effects/
    ├── century-change/  war horn, church bell (a new century starts)
    ├── good-event/      short fanfare
    ├── bad-event/       gong, dark impact
    ├── war-event/       war horn, battle cry
    ├── pact-broken/     dramatic hit, sword unsheathed
    ├── card-trade/      coins dropping, coin pouch
    └── battle-start/    sword clash, army charge
```

## How many

| Folder | Target |
|---|---|
| suspense | 5 |
| battle | 4 |
| dark-age | 3 |
| early-ages, middle-ages, late-ages, good-age, order, intrigue | 2 each |
| war-age, finale | 1 each |
| each effects folder | 2 (century-change and battle-start: 3) |

The page already has 13 tracks. Do not re-download these (Pixabay titles): Just Lute (kaazoom), Squire and Quiet Repose (Medieval Horizons), Medieval Music (Kiravale), Epic Medieval (kulakovka), Medieval Fantasy (RoyaltyFreeMusicStudio), Ironbound March and Crusade of Light (RubyZephyr), The Executioner's Toll (OneCinematicStudio), Steel & Banners (OpenMindAudio), Camelot (Billy Ziogas), and two Warrior Drums tracks (HitsLab).

## Sources and licence

- **Pixabay** (pixabay.com, Music and Sound Effects): preferred. No attribution needed, but keep the artist name for the credits.
- **Freesound** (freesound.org): only files under **Creative Commons 0**.
- **Kevin MacLeod** (incompetech.com): music only. Note in the manifest that he must be credited.
- Nothing from YouTube rips, film or game soundtracks, or anything without a clear free licence.

Keep the original downloaded file names: they contain the artist.

## Checks for every file

1. Format MP3 or M4A, under **15 MB**.
2. Music: 1 to 4 minutes, no long silence at the start or end, no voice-over or watermark. Effects: under 5 seconds, with the sound starting within 0.1 s.
3. If `ffmpeg` is available, measure integrated loudness: `ffmpeg -hide_banner -nostats -i FILE -af ebur128 -f null - 2>&1 | grep "I:" | tail -1`.
4. Fits its folder's description. If unsure, say so in the notes. Do not guess silently.

## Deliverable

A file `Chronicler/manifest.csv`, one row per file:

```
folder,file,title,artist,source,licence,duration_s,size_mb,lufs,notes
music/suspense,tension-medieval-12345.mp3,Tension Medieval,SomeArtist,Pixabay,Pixabay Content License,142,4.1,-14.2,slow build; good for placing troops
```

Then tell Nikita which folders are complete, and which are short and why.

## Hand back

Nikita sends the page-building session the manifest plus the files, one or two folders per message. Files over 15 MB, or any whose licence is unclear, should be flagged, not sent.

## Optional: add the files to the page yourself

Only if your tools include the built-in **Artifact** tool. Otherwise stop at the manifest.

1. Ask Nikita for The Chronicler's artifact link. Read it first with the Artifact tool (`action: "read"`, `url`). You get the page HTML; save it as your working file.
2. Give each new file a short key (lowercase letters and digits, for example `suspense3`). Music is served at `music/<key>.mp3`; effects at `sfx/<key>.mp3`.
3. In the page's script, edit only these constants:
   - `TRACKS`: add `key: ['Title', trimDb]`, where `trimDb = -16 - LUFS`, capped between -8 and +4.
   - `MOODS`: add the key to the matching playlist's `tracks` list. Folder to playlist: suspense→suspense, battle→battle, early-ages→early, middle-ages→middle, late-ages→late, good-age→festive, order→sacred, intrigue→intrigue, dark-age→sombre, war-age→war, finale→finale.
   - `SFX`: add `'sfx/<key>.mp3'` to the list for its kind. Folder to kind: century-change→century, good-event→good, bad-event→bad, war-event→war, pact-broken→betray, card-trade→trade, battle-start→battle (also add `battle: []` to `SFX` and call `sfx('battle')` in `setPhase` when entering Battle).
   - Credits line: add each new artist.
4. Publish with the Artifact tool, passing the same `url`, the edited page as `file_path`, and the new audio as `files` (`{"music/suspense3.mp3": "<local path>", ...}`). At most 255 files and 64 MB per publish, so send big batches in several publishes. Files you leave out are kept.
5. Tell Nikita what you added, so the page-building session can re-read the artifact before its next change.

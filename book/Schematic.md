# `SchDoc` and `SchLib`

## `SchLib`

A schematic library (`.SchLib`) is an OLE file with the following contents:

```text
/
    Component Name/    <-- one entry per component
        Data
        PinFrac
        PinTextData
    Storage
    FileHeader
```


### `FileHeader`

Contains info about the file itself. Contents:

- 4-byte LE content length signature. Seems to always be stream_len - 4
- Content map
- Ending 0x00 byte (included in length)

It _sems_ like the length signature is always accurate, but haven't verified
completely.

The content map is an [`AltiumMap`](Common.md#altiummap).

```text

|HEADER=Protel for Windows - Schematic Library Editor Binary File Version 5.0
|Weight=129
|MinorVersion=3
|UniqueID=RFOIKHCI
|FontIdCount=4
|Size1=10
|FontName1=Times New Roman
|Size2=8
|FontName2=Calibri
```

### `Data`

This contains binary data, sort of like this:

```
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  07 01 00 00 7C 52 45 43  4F 52 44 3D 31 7C 4C 69  ....|RECORD=1|Li
00000010  62 52 65 66 65 72 65 6E  63 65 3D 50 69 6E 5F 50  bReference=Pin_P
00000020  72 6F 70 65 72 74 69 65  73 7C 50 61 72 74 43 6F  roperties|PartCo
00000030  75 6E 74 3D 32 7C 44 69  73 70 6C 61 79 4D 6F 64  unt=2|DisplayMod
00000040  65 43 6F 75 6E 74 3D 31  7C 49 6E 64 65 78 49 6E  eCount=1|IndexIn
00000050  53 68 65 65 74 3D 2D 31  7C 4F 77 6E 65 72 50 61  Sheet=-1|OwnerPa
00000060  72 74 49 64 3D 2D 31 7C  43 75 72 72 65 6E 74 50  rtId=-1|CurrentP
00000070  61 72 74 49 64 3D 31 7C  4C 69 62 72 61 72 79 50  artId=1|LibraryP
00000080  61 74 68 3D 2A 7C 53 6F  75 72 63 65 4C 69 62 72  ath=*|SourceLibr
00000090  61 72 79 4E 61 6D 65 3D  2A 7C 53 68 65 65 74 50  aryName=*|SheetP
000000A0  61 72 74 46 69 6C 65 4E  61 6D 65 3D 2A 7C 54 61  artFileName=*|Ta
000000B0  72 67 65 74 46 69 6C 65  4E 61 6D 65 3D 2A 7C 55  rgetFileName=*|U
000000C0  6E 69 71 75 65 49 44 3D  50 49 41 48 4C 4F 5A 50  niqueID=PIAHLOZP
000000D0  7C 41 72 65 61 43 6F 6C  6F 72 3D 31 31 35 39 39  |AreaColor=11599
000000E0  38 37 31 7C 43 6F 6C 6F  72 3D 31 32 38 7C 50 61  871|Color=128|Pa
000000F0  72 74 49 44 4C 6F 63 6B  65 64 3D 54 7C 41 6C 6C  rtIDLocked=T|All
00000100  50 69 6E 43 6F 75 6E 74  3D 38 00 29 00 00 01 02  PinCount=8.)....
00000110  00 00 00 00 01 00 00 00  00 00 00 00 01 04 38 0A  ..............8.
00000120  00 00 00 00 00 00 00 00  00 06 4E 6F 72 6D 61 6C  ..........Normal
00000130  01 31 00 03 7C 26 7C 00  29 00 00 01 02 00 00 00  .1..|&|.).......
00000140  00 01 00 00 00 00 00 00  00 01 04 38 32 00 00 00  ...........82...
00000150  F6 FF 00 00 00 00 06 4C  65 6E 35 30 30 01 32 00  .......Len500.2.
00000160  03 7C 26 7C 00 29 00 00  01 02 00 00 00 00 01 00  .|&|.)..........
00000170  00 03 00 00 00 00 01 04  38 0A 00 00 00 EC FF 00  ........8.......
00000180  00 00 00 06 43 6C 6B 53  79 6D 01 33 00 03 7C 26  ....ClkSym.3..|&
00000190  7C 00 2D 00 00 01 02 00  00 00 00 01 00 00 00 01  |.-.............
000001A0  00 00 00 01 04 38 0A 00  00 00 E2 FF 00 00 00 00  .....8..........
000001B0  0A 4F 75 74 73 69 64 65  44 6F 74 01 34 00 03 7C  .OutsideDot.4..|
000001C0  26 7C 00 46 00 00 01 02  00 00 00 00 01 00 00 00  &|.F............
000001D0  00 00 00 15 54 68 69 73  20 69 73 20 61 20 64 65  ....This is a de
000001E0  73 63 72 69 70 74 69 6F  6E 01 04 38 0A 00 00 00  scription..8....
000001F0  D8 FF 00 00 00 00 0E 48  61 73 44 65 73 63 72 69  .......HasDescri
00000200  70 74 69 6F 6E 01 35 00  03 7C 26 7C 00 31 00 00  ption.5..|&|.1..
00000210  01 02 00 00 00 00 01 00  00 00 00 00 00 00 01 04  ................
00000220  38 0A 00 00 00 CE FF 00  00 00 00 0E 53 6D 61 6C  8...........Smal
00000230  6C 4C 69 6E 65 57 69 64  74 68 01 36 00 03 7C 26  lLineWidth.6..|&
00000240  7C 00 33 00 00 01 02 00  00 00 00 01 00 00 00 00  |.3.............
00000250  00 00 00 01 04 28 0A 00  00 00 C4 FF 00 00 00 00  .....(..........
00000260  10 48 69 64 64 65 6E 44  65 73 69 67 6E 61 74 6F  .HiddenDesignato
00000270  72 01 37 00 03 7C 26 7C  00 38 00 00 01 02 00 00  r.7..|&|.8......
00000280  00 00 01 00 00 00 00 00  00 00 01 04 30 0A 00 00  ............0...
00000290  00 BA FF 00 00 00 00 0A  48 69 64 64 65 6E 4E 61  ........HiddenNa
000002A0  6D 65 0C 28 48 69 64 64  65 6E 4E 61 6D 65 29 00  me.(HiddenName).
000002B0  03 7C 26 7C 00 95 00 00  00 7C 52 45 43 4F 52 44  .|&|.....|RECORD
000002C0  3D 33 34 7C 49 6E 64 65  78 49 6E 53 68 65 65 74  =34|IndexInSheet
000002D0  3D 2D 31 7C 4F 77 6E 65  72 50 61 72 74 49 64 3D  =-1|OwnerPartId=
000002E0  2D 31 7C 4C 6F 63 61 74  69 6F 6E 2E 58 3D 2D 35  -1|Location.X=-5
000002F0  7C 4C 6F 63 61 74 69 6F  6E 2E 59 3D 35 7C 43 6F  |Location.Y=5|Co
00000300  6C 6F 72 3D 38 33 38 38  36 30 38 7C 46 6F 6E 74  lor=8388608|Font
00000310  49 44 3D 32 7C 54 65 78  74 3D 2A 7C 4E 61 6D 65  ID=2|Text=*|Name
00000320  3D 44 65 73 69 67 6E 61  74 6F 72 7C 52 65 61 64  =Designator|Read
00000330  4F 6E 6C 79 53 74 61 74  65 3D 31 7C 55 6E 69 71  OnlyState=1|Uniq
00000340  75 65 49 44 3D 45 4B 4F  55 44 44 4D 54 00 8F 00  ueID=EKOUDDMT...
00000350  00 00 7C 52 45 43 4F 52  44 3D 34 31 7C 49 6E 64  ..|RECORD=41|Ind
00000360  65 78 49 6E 53 68 65 65  74 3D 2D 31 7C 4F 77 6E  exInSheet=-1|Own
00000370  65 72 50 61 72 74 49 64  3D 2D 31 7C 4C 6F 63 61  erPartId=-1|Loca
00000380  74 69 6F 6E 2E 58 3D 2D  35 7C 4C 6F 63 61 74 69  tion.X=-5|Locati
00000390  6F 6E 2E 59 3D 2D 31 35  7C 43 6F 6C 6F 72 3D 38  on.Y=-15|Color=8
000003A0  33 38 38 36 30 38 7C 46  6F 6E 74 49 44 3D 32 7C  388608|FontID=2|
000003B0  49 73 48 69 64 64 65 6E  3D 54 7C 54 65 78 74 3D  IsHidden=T|Text=
000003C0  2A 7C 4E 61 6D 65 3D 43  6F 6D 6D 65 6E 74 7C 55  *|Name=Comment|U
000003D0  6E 69 71 75 65 49 44 3D  5A 49 4C 51 48 57 55 48  niqueID=ZILQHWUH
000003E0  00 0B 00 00 00 7C 52 45  43 4F 52 44 3D 34 34 00  .....|RECORD=44.
```

It seems like the pattern is:

- 3-byte length (`len`)
- One byte type indicator (seems to be 0x00 for utf8, 0x01 for pins)
- Record string (utf8) of length `len-1`, OR record binary
- Null terminator (brings total length to `len`)

And the above repeats. 

Records start with an ID that identifies what it is representing.

#### Pins

There is a lot of unknown stuff here still, but I'm slowly piecing it
together.

- 6 bytes; unknown (owner index and part ID?) usually 0x020000000001
- 6 more bytes for options including:
  - circle
  - clock symbol
  - arrows
  These will take some trial and error to figure out.
- 1 byte `desc_len`, bytes of the pin description
- `desc_len` bytes utf8 description
- 1 byte "formal type", always 1 (`f`)
- 1 byte type info (`t`)
  - 0: input
  - 1: I/O
  - 2: out
  - 3: open collector
  - 4: passive
  - 5: HiZ
  - 6: OE
  - 7 power
- 1 byte rotation & hiding (`r`)
- 4 bytes pin length (*10) (`l`)
- 4 bytes x position (16bit signed) (`x`)
- 4 bytes y position (16bit signed) (`y`)
- 4 bytes unknown (seems to always be zeroes)
- 1 byte `name_len`
- `name_len` bytes of the pin name
- 1 byte `desig_len` bytes in the designator
- `desig_len` bytes of the designator
- 0x037c267c, the `|&|` pattern. Not sure what this means but it always ends the
  pin.

Presumably some of the missing things are:

- Color
- Line width
- Offsets and fonts of designator and name

A lot of this comes from the old ASCII spec, described here:
https://github.com/vadmium/python-altium/blob/master/format.md#pin

Example from `CombinedPins`

```
 0 1 2 3 4 5 6 7 8 9 a b c [...]         0 1 2 3 4 5 6 7 8 9 0 a b  c [...]
            [opt c     ] s- [pin descr ] f-t-r-l---x---y---         s- [name            ] s- [desig ]
020000000001000000000000 00              01073a0a000a00000000000000 04 [Pin1            ] 01 [1]          00037c267c
020000000001000000000000 00              01073a0a000a00f6ff00000000 04 [Pin2            ] 01 [2]          00037c267c
020000000001000000000000 00              0107380a004600000000000000 04 [Pin3            ] 01 [3]          00037c267c
020000000001000000000000 00              0107380a004600f6ff00000000 04 [Pin4            ] 01 [4]          00037c267c
020000000001000000000000 00              0104380a000000000000000000 07 [PINNAME         ] 06 [PINDES]     00037c267c
020000000001000000000000 00              0104380a000000000000000000 09 [Pin (0,0)       ] 06 [PINDES]     00037c267c
020000000001000000000000 00              0104380a000000140000000000 0b [Pin (0,200)     ] 06 [PINDES]     00037c267c
020000000001000000000000 00              0104380a006e00000000000000 0c [Pin (1100,0)    ] 06 [PINDES]     00037c267c
020000000001000000000000 00              0104380a002800ecff00000000 06 [Pin R0          ] 03 [DES]        00037c267c
020000000001000000000000 00              0104390a006e00c4ff00000000 07 [Pin R90         ] 03 [DES]        00037c267c
020000000001000000000000 00              01043a0a00d8ffecff00000000 08 [Pin R180        ] 03 [DES]        00037c267c
020000000001000000000000 00              01043b0a005a00baff00000000 08 [Pin R270        ] 03 [DES]        00037c267c
020000000001000000000000 00              0100380a00f6ffc4ff00000000 05 [Input           ] 01 [1]          00037c267c
020000000001000000000000 00              0101380a00f6ffbaff00000000 03 [I/O             ] 01 [2]          00037c267c
020000000001000000000000 00              0102380a00f6ffb0ff00000000 03 [Out             ] 01 [3]          00037c267c
020000000001000000000000 00              0103380a00f6ffa6ff00000000 02 [OC              ] 01 [4]          00037c267c
020000000001000000000000 00              0104380a00f6ff9cff00000000 07 [Passive         ] 01 [5]          00037c267c
020000000001000000000000 00              0105380a00f6ff92ff00000000 03 [HiZ             ] 01 [6]          00037c267c
020000000001000000000000 00              0106380a00f6ff88ff00000000 02 [OE              ] 01 [7]          00037c267c
020000000001000000000000 00              0107380a00f6ff7eff00000000 05 [Power           ] 01 [8]          00037c267c
020000000001000000000000 00              0104380A000000000000000000 06 [Normal          ] 01 [1]          00037C267C
020000000001000000000000 00              01043832000000F6FF00000000 06 [Len500          ] 01 [2]          00037C267C
020000000001000003000000 00              0104380A000000ECFF00000000 06 [ClkSym          ] 01 [3]          00037C267C
020000000001000000010000 00              0104380A000000E2FF00000000 0A [OutsideDot      ] 01 [4]          00037C267C
020000000001000000000000 15 [This is...] 0104380A000000D8FF00000000 0E [HasDescription  ] 01 [5]          00037C267C
020000000001000000000000 00              0104380A000000CEFF00000000 0E [SmallLineWidth  ] 01 [6]          00037C267C
020000000001000000000000 00              0104280A000000C4FF00000000 10 [HiddenDesignator] 01 [7]          00037C267C
020000000001000000000000 00              0104300A000000BAFF00000000 0A [HiddenName      ] 0C [HiddenName] 00037C267C
020000000001000000000000 00              0107380A00F6FF000000000000 07 [Calibri         ] 02 [NA]         00037C267C
020000000001000000000000 00              0107380A00F6FFF6FF00000000 05 [Arial           ] 02 [NA]         00037C267C
020000000001000000000000 00              0107380A00F6FFECFF00000000 07 [500 gap         ] 02 [NA]         00037C267C
020000000001000000000000 00              0107380A00F6FFE2FF00000000 02 [NA              ] 07 [Calibri]    00037C267C
020000000001000000000000 00              0107380A00F6FFD8FF00000000 02 [NA              ] 05 [Arial]      00037C267C
020000000001000000000000 00              0107380A00F6FFCEFF00000000 02 [NA              ] 07 [500 gap]    00037C267C
020000000001000000000000 00              0107380A00F6FF92FF00000000 02 [NA              ] 0D [90Ori..Pin] 00037C267C
020000000001000000000000 00              0107380A00F6FF88FF00000000 02 [NA              ] 0D [0Ori..Part] 00037C267C
020000000001000000000000 00              0107380A00F6FF74FF00000000 02 [NA              ] 11 [Nam.0.Part] 00037C267C
020000000001000000000000 00              0107380A00F6FF6AFF00000000 02 [NA              ] 12 [Nam90.Part] 00037C267C0095000000
020000000001000000000000 00              0107380A000000000000000000 01 [1               ] 01 [1]          00037C267C
020000000001000000000000 00              0107380A000000F6FF00000000 01 [2               ] 01 [2]          00037C267C00B3000000
```

### `Storage`

It seems like the `Storage` is the following:

- u32 `header length`
- `header length` bytes of something like `|HEADER=Icon storage|Weight=6`,
  includes null terminator. `Weight` seems to be the number of stored objects.
  `Weight` is not included if it is empty.
- Repeating:
  - 4 bytes `content length`. It seems like, as with pins, the upper bit is set
    to `1` to indicate binary storage (e.g. `DD 5C 02 01` and `A5 02 00 01`) 
  - A byte `D0` that seems to always be the same. Magic?
  - One byte length of the path name
  - The path name, _no_ null termination
  - 4 bytes u32 `payload length`
  - `payload length` bytes of data, zlib compressed

Interesting that it seems like the length of the whole thing is encoded twice in
both `content length` and `payload length`.

That was a lucky thing to figure out, I just happened to notice the bytes `78
9C` were in every file, and the magic bytes list happened to list that as zstd
default compression.

It seems like after extracting the data, there is both a BMP and JPEG together? I am unsure.

![image](https://github.com/pluots/altium-rs/assets/13724985/2dc948fc-c77b-43cd-89d6-6c762d6148c9)


There is also some weirdness here. It seems like Altium changes images with
transparant backgrounds to white? We manually make them transparant again, but
it's weird that we have to do this.

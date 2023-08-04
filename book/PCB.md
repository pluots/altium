# `PCBDoc` and `PCBLib`

## `PcbLib`

```text
/
    Component Name/                   <-- one entry per component
        Data
        Header
        Parameters
        WideStrings
        UniqueIDPrimitiveInformation  <-- optional
    FileVersionInfo/
        Data
        Header
    Library/
        ComponentParamsTOC/
            Data
            Header
        LayerKindMapping/
            Data
            Header
        Models/
            Data
            Header
            0                           <-- Numbered streams are optional
            1
        ModelsNoEmbed/
            Data
            Header
        PadViaLibrary/
            Data
            Header
        Textures/
            Data
            Header
        Data
        EmbeddedFonts
        Header
    FileHeader
```

### `FileHeader`

Starts with a 4 byte length, then the same length (?) followed by a header,
followed by something meaningless. Seems to always be the same.

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  1B 00 00 00 1B 50 43 42  20 36 2E 30 20 42 69 6E  .....PCB 6.0 Bin
00000010  61 72 79 20 4C 69 62 72  61 72 79 20 46 69 6C 65  ary Library File
00000020  0A D7 A3 70 3D 0A 14 40  08 00 00 00 08 4E 46 52  ...p=..@.....NFR
00000030  51 52 54 4C 4E                                   QRTLN
```

### `Library`

This seems to contain metadata about the entire PcbLib

#### `ComponentParamsTOC`

`Header` is empty, `Data` 4 byte length followed by the contents. Contents seems
to be a KV map of basic information for each component, with a CRLF separator
between components (0x0d 0x0a). We can probably use this to extract an overview
of what this library contains.

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  BD 04 00 00 4E 61 6D 65  3D 46 6F 75 72 20 70 61  ....Name=Four pa
00000010  64 73 7C 50 61 64 20 43  6F 75 6E 74 3D 34 7C 48  ds|Pad Count=4|H
00000020  65 69 67 68 74 3D 33 39  2E 33 37 30 31 7C 44 65  eight=39.3701|De
00000030  73 63 72 69 70 74 69 6F  6E 3D 46 6F 75 72 20 70  scription=Four p
00000040  61 64 73 20 6F 66 20 64  69 66 66 65 72 65 6E 74  ads of different
00000050  20 74 79 70 65 73 0D 0A  4E 61 6D 65 3D 46 6F 6F   types..Name=Foo
00000060  74 70 72 69 6E 74 20 32  32 7C 50 61 64 20 43 6F  tprint 22|Pad Co
00000070  75 6E 74 3D 31 7C 48 65  69 67 68 74 3D 33 39 2E  unt=1|Height=39.
00000080  33 37 30 31 7C 44 65 73  63 72 69 70 74 69 6F 6E  3701|Description
00000090  3D 46 6F 6F 74 70 72 69  6E 74 20 32 0D 0A 4E 61  =Footprint 2..Na
000000A0  6D 65 3D 53 69 6E 67 6C  65 20 50 61 64 20 28 30  me=Single Pad (0
000000B0  2C 30 29 7C 50 61 64 20  43 6F 75 6E 74 3D 31 7C  ,0)|Pad Count=1|
000000C0  48 65 69 67 68 74 3D 33  39 2E 33 37 30 31 7C 44  Height=39.3701|D
000000D0  65 73 63 72 69 70 74 69  6F 6E 3D 50 61 64 20 61  escription=Pad a
000000E0  74 20 28 30 2C 30 29 20  64 69 6D 73 20 31 2E 35  t (0,0) dims 1.5
000000F0  78 31 2E 35 20 6D 6D 0D  0A 4E 61 6D 65 3D 43 41  x1.5 mm..Name=CA
00000100  50 43 31 36 30 38 58 30  39 4C 7C 50 61 64 20 43  PC1608X09L|Pad C
00000110  6F 75 6E 74 3D 32 7C 48  65 69 67 68 74 3D 33 35  ount=2|Height=35
00000120  2E 34 33 33 31 7C 44 65  73 63 72 69 70 74 69 6F  .4331|Descriptio
00000130  6E 3D 43 68 69 70 20 43  61 70 61 63 69 74 6F 72  n=Chip Capacitor
00000140  2C 20 32 2D 4C 65 61 64  73 2C 20 42 6F 64 79 20  , 2-Leads, Body 
00000150  31 2E 35 35 78 30 2E 38  30 6D 6D 2C 20 49 50 43  1.55x0.80mm, IPC
```

#### `LayerKindMapping`

`Header` is empty, `Data` is binary.

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  08 00 00 00 31 00 2E 00  30 00 00 00 3E DB A8 F4  ....1...0...>...
00000010  02 00 00 00 48 00 00 00  0C 00 00 00 47 00 00 00  ....H.......G...
00000020  0B 00 00 00                                      ....
```

#### `Models`

Header is empty (well, this sample starts with 0x02 while others start with
0x01...). `Data` contains repeating KV maps (u32 length, null-terminated string)
that describe each model (presumably only embedded?). Seems to contain model
metadata including a v4 UUID. I wonder if there may be one entry per use rather
than one entry per model, since rotation is described here.

```
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  9C 00 00 00 45 4D 42 45  44 3D 54 52 55 45 7C 4D  ....EMBED=TRUE|M
00000010  4F 44 45 4C 53 4F 55 52  43 45 3D 55 6E 64 65 66  ODELSOURCE=Undef
00000020  69 6E 65 64 7C 49 44 3D  7B 30 42 39 38 44 30 35  ined|ID={0B98D05
00000030  43 2D 35 34 36 41 2D 34  36 41 42 2D 41 42 34 35  C-546A-46AB-AB45
00000040  2D 45 39 32 31 42 46 45  39 36 32 44 43 7D 7C 52  -E921BFE962DC}|R
00000050  4F 54 58 3D 30 2E 30 30  30 7C 52 4F 54 59 3D 30  OTX=0.000|ROTY=0
00000060  2E 30 30 30 7C 52 4F 54  5A 3D 30 2E 30 30 30 7C  .000|ROTZ=0.000|
00000070  44 5A 3D 30 7C 43 48 45  43 4B 53 55 4D 3D 31 36  DZ=0|CHECKSUM=16
00000080  38 37 30 32 34 33 37 31  7C 4E 41 4D 45 3D 43 41  87024371|NAME=CA
00000090  50 43 31 36 30 38 58 30  39 4C 2E 73 74 65 70 00  PC1608X09L.step.
000000A0  AA 00 00 00 45 4D 42 45  44 3D 54 52 55 45 7C 4D  ....EMBED=TRUE|M
000000B0  4F 44 45 4C 53 4F 55 52  43 45 3D 55 6E 64 65 66  ODELSOURCE=Undef
000000C0  69 6E 65 64 7C 49 44 3D  7B 45 39 45 37 32 32 35  ined|ID={E9E7225
000000D0  39 2D 45 30 45 39 2D 34  31 38 46 2D 41 38 38 42  9-E0E9-418F-A88B
000000E0  2D 32 36 44 30 46 30 34  35 46 31 44 44 7D 7C 52  -26D0F045F1DD}|R
000000F0  4F 54 58 3D 30 2E 30 30  30 7C 52 4F 54 59 3D 30  OTX=0.000|ROTY=0
00000100  2E 30 30 30 7C 52 4F 54  5A 3D 30 2E 30 30 30 7C  .000|ROTZ=0.000|
00000110  44 5A 3D 30 7C 43 48 45  43 4B 53 55 4D 3D 2D 31  DZ=0|CHECKSUM=-1
00000120  39 38 33 36 31 33 30 35  33 7C 4E 41 4D 45 3D 53  983613053|NAME=S
00000130  51 46 50 35 30 50 38 30  30 58 38 30 30 58 33 30  QFP50P800X800X30
00000140  30 5F 48 53 2D 33 33 4E  2E 73 74 65 70 00        0_HS-33N.step.
```

This storage also contains numbered files (`0`, `1`, etc) that likely line up
with this header. These are zlib-compressed step files with the header
`ISO-10303-21` (maybe other types are allowed?), which can be extracted with
this one liner:

```sh
python -c 'import zlib, sys; sys.stdout.buffer.write(zlib.decompress(open("path/1", "rb").read()))'

# or, same thing, gunzip just needs this header
printf "\x1f\x8b\x08\x00\x00\x00\x00\x00" | cat - path/1 | gunzip
```

#### `ModelsNoEmbed`

Both `Data` and `Header` seem empty when unused.

#### `PadViaLibrary`

`Data` has information, `Header` is just null u32.

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  7F 00 00 00 7C 50 41 44  56 49 41 4C 49 42 52 41  ....|PADVIALIBRA
00000010  52 59 2E 4C 49 42 52 41  52 59 49 44 3D 7B 45 38  RY.LIBRARYID={E8
00000020  31 31 30 31 30 32 2D 43  41 36 36 2D 34 30 39 31  110102-CA66-4091
00000030  2D 42 46 33 37 2D 39 32  31 31 42 34 44 31 36 43  -BF37-9211B4D16C
00000040  41 33 7D 7C 50 41 44 56  49 41 4C 49 42 52 41 52  A3}|PADVIALIBRAR
00000050  59 2E 4C 49 42 52 41 52  59 4E 41 4D 45 3D 3C 4C  Y.LIBRARYNAME=<L
00000060  6F 63 61 6C 3E 7C 50 41  44 56 49 41 4C 49 42 52  ocal>|PADVIALIBR
00000070  41 52 59 2E 44 49 53 50  4C 41 59 55 4E 49 54 53  ARY.DISPLAYUNITS
00000080  3D 31 00                                         =1.
```

#### `Textures`

`Data` is empty when unused, `Header` is a null `u32`. Not sure what this is
used for.

#### `Data`

The top-level `Data` seems to contain the library stackup and some metadata.


```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  1D 72 01 00 7C 46 49 4C  45 4E 41 4D 45 3D 43 3A  .r..|FILENAME=C:
00000010  5C 55 73 65 72 73 5C 74  67 72 6F 73 73 5C 44 6F  \Users\tgross\Do
00000020  63 75 6D 65 6E 74 73 5C  50 72 6F 67 72 61 6D 6D  cuments\Programm
00000030  69 6E 67 5C 50 79 41 6C  74 69 75 6D 5C 74 65 73  ing\PyAltium\tes
00000040  74 5C 66 69 6C 65 73 5C  50 63 62 4C 69 62 31 2E  t\files\PcbLib1.
00000050  24 24 24 7C 4B 49 4E 44  3D 50 72 6F 74 65 6C 5F  $$$|KIND=Protel_
00000060  41 64 76 61 6E 63 65 64  5F 50 43 42 5F 4C 69 62  Advanced_PCB_Lib
00000070  72 61 72 79 7C 56 45 52  53 49 4F 4E 3D 33 2E 30  rary|VERSION=3.0
00000080  30 7C 44 41 54 45 3D 32  30 32 32 2D 30 31 2D 30  0|DATE=2022-01-0
00000090  36 7C 54 49 4D 45 3D 32  31 3A 33 36 3A 30 39 7C  6|TIME=21:36:09|
000000A0  56 39 5F 4D 41 53 54 45  52 53 54 41 43 4B 5F 53  V9_MASTERSTACK_S
000000B0  54 59 4C 45 3D 30 7C 56  39 5F 4D 41 53 54 45 52  TYLE=0|V9_MASTER
000000C0  53 54 41 43 4B 5F 49 44  3D 7B 38 44 38 46 44 35  STACK_ID={8D8FD5
000000D0  42 39 2D 32 36 38 38 2D  34 45 31 30 2D 42 42 36  B9-2688-4E10-BB6
000000E0  32 2D 42 44 46 30 39 46  42 43 32 33 46 46 7D 7C  2-BDF09FBC23FF}|
000000F0  56 39 5F 4D 41 53 54 45  52 53 54 41 43 4B 5F 4E  V9_MASTERSTACK_N
00000100  41 4D 45 3D 4D 61 73 74  65 72 20 6C 61 79 65 72  AME=Master layer
00000110  20 73 74 61 63 6B 7C 56  39 5F 4D 41 53 54 45 52   stack|V9_MASTER
00000120  53 54 41 43 4B 5F 53 48  4F 57 54 4F 50 44 49 45  STACK_SHOWTOPDIE
00000130  4C 45 43 54 52 49 43 3D  46 41 4C 53 45 7C 56 39  LECTRIC=FALSE|V9
00000140  5F 4D 41 53 54 45 52 53  54 41 43 4B 5F 53 48 4F  _MASTERSTACK_SHO
00000150  57 42 4F 54 54 4F 4D 44  49 45 4C 45 43 54 52 49  WBOTTOMDIELECTRI
00000160  43 3D 46 41 4C 53 45 7C  56 39 5F 4D 41 53 54 45  C=FALSE|V9_MASTE
00000170  52 53 54 41 43 4B 5F 49  53 46 4C 45 58 3D 46 41  RSTACK_ISFLEX=FA
```

#### `EmbeddedFonts`

Empty if unused

### Component

#### `Data`

A bunch of binary representing the component, this will be fun.

It does seem to start with the component name - perhaps this is more reliable
than using the storage name?

#### `Header`

Has a nonzero `u32`, not sure what this is for.

#### `Parameters`

Standard KV with basic metadata.

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  88 00 00 00 7C 50 41 54  54 45 52 4E 3D 43 41 50  ....|PATTERN=CAP
00000010  43 31 36 30 38 58 30 39  4C 7C 48 45 49 47 48 54  C1608X09L|HEIGHT
00000020  3D 33 35 2E 34 33 33 31  6D 69 6C 7C 44 45 53 43  =35.4331mil|DESC
00000030  52 49 50 54 49 4F 4E 3D  43 68 69 70 20 43 61 70  RIPTION=Chip Cap
00000040  61 63 69 74 6F 72 2C 20  32 2D 4C 65 61 64 73 2C  acitor, 2-Leads,
00000050  20 42 6F 64 79 20 31 2E  35 35 78 30 2E 38 30 6D   Body 1.55x0.80m
00000060  6D 2C 20 49 50 43 20 48  69 67 68 20 44 65 6E 73  m, IPC High Dens
00000070  69 74 79 7C 49 54 45 4D  47 55 49 44 3D 7C 52 45  ity|ITEMGUID=|RE
00000080  56 49 53 49 4F 4E 47 55  49 44 3D 00              VISIONGUID=.
```

#### `WideStrings`

Empty on mine, maybe used for something with UTF-16?

#### `UniqueIDPrimitiveInformation`

Not sure what `Data` is, `Header` is empty

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  3A 00 00 00 7C 50 52 49  4D 49 54 49 56 45 49 4E  :...|PRIMITIVEIN
00000010  44 45 58 3D 30 7C 50 52  49 4D 49 54 49 56 45 4F  DEX=0|PRIMITIVEO
00000020  42 4A 45 43 54 49 44 3D  50 61 64 7C 55 4E 49 51  BJECTID=Pad|UNIQ
00000030  55 45 49 44 3D 49 44 58  4F 44 52 4E 45 00        UEID=IDXODRNE.
```

### `Header`

Empty, not sure what this is for

### `FileVersionInfo`

Seems to contain a message to display for each version compatibility match
(COUNT is the total number of versions). The messages are strings encoded to
ASCII, then those numbers are written as ASCII themselves to the blob (like...
why?).

Just pop it into python `bytearray([ <paste data here> ])` to print the value

```text
Hex View  00 01 02 03 04 05 06 07  08 09 0A 0B 0C 0D 0E 0F
 
00000000  09 0A 00 00 7C 43 4F 55  4E 54 3D 35 7C 56 45 52  ....|COUNT=5|VER
00000010  30 3D 38 37 2C 31 30 35  2C 31 31 30 2C 31 31 36  0=87,105,110,116
00000020  2C 31 30 31 2C 31 31 34  2C 33 32 2C 34 38 2C 35  ,101,114,32,48,5
00000030  37 7C 46 57 44 4D 53 47  30 3D 7C 42 4B 4D 53 47  7|FWDMSG0=|BKMSG
00000040  30 3D 36 30 2C 39 38 2C  36 32 2C 36 37 2C 36 35  0=60,98,62,67,65
00000050  2C 38 35 2C 38 34 2C 37  33 2C 37 39 2C 37 38 2C  ,85,84,73,79,78,
00000060  36 30 2C 34 37 2C 39 38  2C 36 32 2C 33 32 2C 34  60,47,98,62,32,4
00000070  35 2C 33 32 2C 38 36 2C  31 30 35 2C 39 37 2C 31  5,32,86,105,97,1
00000080  31 35 2C 33 32 2C 31 31  35 2C 31 31 37 2C 31 31  15,32,115,117,11
00000090  32 2C 31 31 32 2C 31 31  31 2C 31 31 34 2C 31 31  2,112,111,114,11
000000A0  36 2C 33 32 2C 31 31 38  2C 39 37 2C 31 31 34 2C  6,32,118,97,114,
000000B0  31 32 31 2C 31 30 35 2C  31 31 30 2C 31 30 33 2C  121,105,110,103,
000000C0  33 32 2C 31 30 30 2C 31  30 35 2C 39 37 2C 31 30  32,100,105,97,10
000000D0  39 2C 31 30 31 2C 31 31  36 2C 31 30 31 2C 31 31  9,101,116,101,11
000000E0  34 2C 31 31 35 2C 33 32  2C 39 37 2C 39 39 2C 31  4,115,32,97,99,1
000000F0  31 34 2C 31 31 31 2C 31  31 35 2C 31 31 35 2C 33  14,111,115,115,3
```

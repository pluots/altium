A schematic library 

```text
/
    Component Name/ (storage)
        Data
        PinFrac
        PinTextData
    Storage (stream)
    FileHeader (stream)

```

## `FileHeader`

Contains info about the file itself

```text
z
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

## `Storage`

I got _so_ lucky with this one! It seems like this is filled with a repeating storage
along these lines:

```
File name
length: u16
data: [u8; length]
```

And then data is zlib compressed! That was the tricky thing to figure out,
I just happened to notice the bytes `78 9C` were in every file, and the magic bytes list
happened to list that as zstd default compression.

It seems like after extracting the data, there is both a BMP and JPEG together? I am unsure.

![image](https://github.com/pluots/altium-rs/assets/13724985/2dc948fc-c77b-43cd-89d6-6c762d6148c9)


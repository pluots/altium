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

Seems to contain binary data related to the entire library

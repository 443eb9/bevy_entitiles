# What's New:

- Added `RenderChunkSort`.
- prevent panic when loading a .TSX file with custom properties #27
- add a more explicit error message when trying to use primitive properties #29
- when a class or a class property is expected but not found, use its default value #30

# What's Fixed:

- Wrong stacking order for tiles that are rendered larger than the slot. #26
- Ldtk sprite flip is incorrect #25
- fix compilation with 'serializing' feature when 'algorithm' is not used #28

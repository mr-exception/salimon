# Scout habitation kit

Parametric independent fixture templates, authored with Blender 4.5.3. `author.py`
creates `source.blend` using linked Scout materials; `manifest.json` defines exact
names and budgets. Runtime placement/physical source is the checked `geometry.json`,
not inferred from rendered nodes. No external generated assets or textures.

```sh
"$BLENDER" --background --factory-startup --disable-autoexec --python-exit-code 1 --python models/assets/fixtures/scout-habitation-kit/author.py
python models/tools/export_asset.py fixture.scout-habitation-kit --blender "$BLENDER"
python models/assets/fixtures/scout-habitation-kit/validate_templates.py
```

The ten templates total 200 triangles, 10 primitives, three shared materials and
no texture bytes. Template-local generic intersection checks run independently,
because the library deliberately stores all reusable templates at the origin.
The production importer and runtime tests audit mesh/physical bounds. Five shutter
segments separate into depth tracks before stacking at the window header; three
screen panels stack beside the bunk. Table, seat pad and locker door use explicit
rigid hinge origins. Refer to [the runtime contract](../../../../docs/cabin-mechanisms.md).

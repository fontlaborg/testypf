# TestYPF Examples

## render_once

Render a single font using the typf pipeline from the command line.

```bash
# Build typf bindings once (or run ./build.sh --verify)
pushd ../typf/bindings/python
uv venv
source venv/bin/activate
uv pip install maturin -q
maturin develop --features "shaping-hb,render-orge"
popd

# Run the example with a local font file
cargo run --example render_once -- /System/Library/Fonts/Supplemental/Arial.ttf "Hello TestYPF"
```

Notes:
- Ensure `PYTHONPATH` includes `../typf/bindings/python/venv/lib/python*/site-packages` if typf is not discoverable.
- Use `TYPF_FEATURES` to extend feature flags passed to typf during the build.

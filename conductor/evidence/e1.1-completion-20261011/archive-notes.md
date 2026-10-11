# Evidence packaging notes

Original worker context bytes are retained as context.txt, because their relative
Markdown links refer to original source locations, not this evidence directory.
Initial context-link check failure is retained. Packet/context hashes still refer
to the original bytes; no instruction text was changed.

The locked native-success log is preserved byte-for-byte as e0-native-network.log.gz.
Its decompressed SHA-256 matches the native receipt. Compression preserves its
original trailing blank line without failing the staged whitespace check. The
first staged check stopped with exit2 before commit; no gate was bypassed.

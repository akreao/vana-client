# Packet capture

A development aid for comparing servers: the client can record the map server's traffic in both directions.

## Turning it on

- `"packet_capture": true` in `config.json`, or
- `VANA_CAPTURE=1` in the environment (`VANA_CAPTURE=/some/folder` writes there instead).

Each connection writes `<data dir>/vana-client/captures/<unix seconds>.jsonl` (on Linux `~/.local/share/vana-client/captures`).

## What is in a capture

One JSON object a line:

- a sub-packet, with its 4-byte header, as it went out or came in:
  `{"t":1763300000123,"dir":"in","zone":230,"id":143,"hex":"8f2c…"}`
- a datagram: its size in bytes, header sequence and ack, and the coder's form (`huffman`, `plain` for the copied
  form behind a byte other than 1, `raw` for the unencrypted first 0x00A):
  `{"t":…,"dir":"in","zone":230,"dgram":312,"seq":41,"ack":17,"form":"huffman"}`

`t` is Unix time in milliseconds; `zone` is the one the last s2c 0x00A named.

Only the map session is recorded. The lobby, PlayOnline sign-in and patch connections never are, since they carry
the password and account keys. The map login (c2s 0x00A) is written with its account name and ticket zeroed. Chat
and tells are recorded as they arrive, so treat a capture as private and cut it down before sharing it.

## Reading one

```
vana-client capture FILE [--id 08f,0c5] [--dir in|out] [--zone N] [--find 1e09] [--hex] [--save OUT]
```

It prints the kept sub-packets with their time from the start, direction, zone, id and common name and size (`--hex`
adds the bytes), then a count of each id each way and of each datagram form each way. `--find` keeps sub-packets
that contain the given bytes anywhere. `--save` writes the kept lines to a new file, so you can share only the
packets that matter.
